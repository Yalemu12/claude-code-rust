use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
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

    let tools = json!([
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
                            "description": "the path to the file to read"
                        }
                    },
                    "required": ["file_path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "Write",
                "description": "Write content to a file",
                "parameters": {
                    "type": "object",
                    "required": ["file_path", "content"],
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "The path of the file to write to"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to write to the file"
                        }
                    }
                }
            }
        }
    ]);

    let mut messages = vec![
        json!({
            "role": "user",
            "content": args.prompt
        })
    ];

    loop {
        let response: Value = client
            .chat()
            .create_byot(json!({
                "model": "anthropic/claude-haiku-4.5",
                "messages": messages,
                "tools": tools
            }))
            .await?;

        let message = response["choices"][0]["message"].clone();

        messages.push(message.clone());

        let tool_calls = message["tool_calls"].as_array();

        if tool_calls.is_none() || tool_calls.unwrap().is_empty() {
            if let Some(content) = message["content"].as_str() {
                println!("{}", content);
            }
            break;
        }

        for tool_call in tool_calls.unwrap() {
            let name = tool_call["function"]["name"].as_str().unwrap();
            let arguments: Value =
                serde_json::from_str(tool_call["function"]["arguments"].as_str().unwrap())?;
            let tool_call_id = tool_call["id"].as_str().unwrap();

            let result = if name == "Read" {
                let file_path = arguments["file_path"].as_str().unwrap();
                std::fs::read_to_string(file_path).unwrap_or_else(|e| e.to_string())
            } else if name == "Write" {
                let file_path = arguments["file_path"].as_str().unwrap();
                let content = arguments["content"].as_str().unwrap();
                match std::fs::write(file_path, content) {
                    Ok(_) => format!("Successfully wrote to {}", file_path),
                    Err(e) => e.to_string(),
                }
            } else {
                format!("Unknown tool: {}", name)
            };

            messages.push(json!({
                "role": "tool",
                "tool_call_id": tool_call_id,
                "content": result
            }));
        }
    }

    Ok(())
}