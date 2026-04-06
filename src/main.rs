use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{env, fs, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionCall,
}

#[derive(Serialize, Deserialize, Clone)]
struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "role")]
enum Message {
    #[serde(rename = "system")]
    System { content: String },
    #[serde(rename = "assistant")]
    Assistant {
        content: Option<String>,
        tool_calls: Option<Vec<ToolCall>>,
    },
    #[serde(rename = "user")]
    User { content: String },
    #[serde(rename = "tool")]
    Tool {
        content: String,
        tool_call_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);

    let mut messages: Vec<Message> = vec![Message::User {
        content: args.prompt,
    }];

    loop {
        let response: Value = client
            .chat()
            .create_byot(json!({
                "messages": messages,
                "model": "anthropic/claude-haiku-4.5",
                "tools": [
                    {
                        "type": "function",
                        "function": {
                            "name": "Read",
                            "description": "Read and return the contents of a file",
                            "parameters": {
                                "type": "object",
                                "properties": {
                                    "file_path": {
                                        "type": "string",
                                        "description": "The path to the file to read"
                                    }
                                },
                                "required": ["file_path"]
                            }
                        }
                    }
                ],
            }))
            .await?;

        let raw_msg = &response["choices"][0]["message"];
        let message: Message = serde_json::from_value(raw_msg.clone())?;

        messages.push(message.clone());

        match message {
            Message::Assistant {
                content,
                tool_calls,
            } => {
                if let Some(tool_calls) = tool_calls {
                    let mut results: Vec<Message> = tool_calls
                        .iter()
                        .map(|call| {
                            let id = call.id.clone();
                            let f_name = call.function.name.as_str();
                            let f_args = call.function.arguments.as_str();

                            let content = (|| {
                                let args_json: serde_json::Value = serde_json::from_str(f_args)
                                    .map_err(|e| format!("Invalid JSON args: {}", e))?;

                                match f_name {
                                    "Read" => {
                                        let file_path = args_json
                                            .get("file_path")
                                            .and_then(|v| v.as_str())
                                            .ok_or("Missing required argument: file_path")?;

                                        let contents = fs::read_to_string(file_path)
                                            .map_err(|e| format!("Error reading file: {}", e))?;

                                        Ok(contents)
                                    }

                                    _ => Ok(format!("Unknown tool: {}", f_name)),
                                }
                            })()
                            .unwrap_or_else(|e: String| e);

                            Message::Tool {
                                tool_call_id: id,
                                content,
                            }
                        })
                        .collect();

                    messages.append(&mut results);
                } else if let Some(content) = content {
                    println!("{}", content);
                    break;
                }
            }
            _ => {
                panic!("Unexpected message type");
            }
        }
    }

    Ok(())
}
