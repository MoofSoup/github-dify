use maud::{html, Markup, DOCTYPE};
use axum::{Router, routing::get, Json, routing::post};
use dify_client::{request, response::ChatMessagesResponse,  Client, Config, response::WorkflowsRunResponse, response::WorkflowFinishedData};
use dify_client::request::WorkflowsRunRequest;
use std::time::Duration;
use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
// use serde_json::Value as JsonValue;

fn debug_workflow_result(result: &WorkflowFinishedData) -> String {
    let mut debug_info = String::new();

    debug_info.push_str(&format!("Status: {:?}\n", result.status));
    debug_info.push_str(&format!("Error: {:?}\n", result.error));
    debug_info.push_str(&format!("Total tokens: {:?}\n", result.total_tokens));
    debug_info.push_str(&format!("Elapsed time: {:?}\n", result.elapsed_time));
    debug_info.push_str(&format!("Total steps: {}\n", result.total_steps));
    debug_info.push_str(&format!("Created at: {}\n", result.created_at));
    debug_info.push_str(&format!("Finished at: {}\n", result.finished_at));

    debug_info.push_str("\nOutputs:\n");
    match &result.outputs {
        Some(outputs) => {
            debug_info.push_str(&format!("Raw outputs: {:?}\n", outputs));
            debug_info.push_str("Available keys in outputs:\n");
            /*for key in outputs.keys() {
                debug_info.push_str(&format!("- {}\n", key));
            }*/
            if let Some(result_value) = outputs.get("result") {
                debug_info.push_str(&format!("\nValue of 'result' key: {:?}\n", result_value));
                if let Some(result_str) = result_value.as_str() {
                    debug_info.push_str(&format!("'result' as string: {}\n", result_str));
                } else {
                    debug_info.push_str("'result' is not a string\n");
                }
            } else {
                debug_info.push_str("No 'result' key found in outputs\n");
            }
        },
        None => debug_info.push_str("No outputs available\n"),
    }

    debug_info.push_str("\nExtra fields:\n");
    for (key, value) in &result.extra {
        debug_info.push_str(&format!("{}: {:?}\n", key, value));
    }

    debug_info
}

async fn question() -> Result<WorkflowsRunResponse> {
    let api_key = env::var("DIFY_API_KEY").expect("DIFY_API_KEY must be set in the .env file");
    let config = Config {
        base_url: "https://api.dify.ai".into(),
        api_key: api_key,
        timeout: Duration::from_secs(60),
    };
    let client = Client::new_with_config(config);

    // Use the client
    
    let input_text:String =  "What's the best City for tech? What's the best college for someone living there to pursue programming on a budget, who's just starting school? I would like community college options. Also, who can I network with on campus maximize my chances? What should I do while networking in the event that the economy is not conducive towards internships? Be specific.".into();
    let mut input_map = HashMap::new();
    input_map.insert("meow".to_string(), input_text);


    let data = WorkflowsRunRequest {
        inputs: input_map,
        response_mode: request::ResponseMode::Blocking,
        user: "Moof".into(),
        files:Vec::new(),

    };

    let result = client.api().workflows_run(data).await;
    Ok(result.unwrap())
}

fn extract_json_response(result: &WorkflowFinishedData) -> Option<Value> {
    result.outputs.as_ref().and_then(|outputs| {
        outputs.get("json response").and_then(|json_str| {
            json_str.as_str().and_then(|str_value| {
                serde_json::from_str(str_value).ok()
            })
        })
    })
}

///
/// 
/// this is where the new code starts from that parses the post request
/// 
/// 
/// 
/// 
///
#[derive(Deserialize)]
struct ChoicesRequest {
    #[serde(rename = "To Do List")]
    to_do_list: String,
    #[serde(rename = "Daily Schedule")]
    daily_schedule: String,
}

#[derive(Serialize, Deserialize)]
struct ChoicesResponse {
    life_tasks: HashMap<String, String>,
    work_tasks: HashMap<String, String>,
}

async fn choices(
    Json(input): Json<ChoicesRequest>,
) -> Json<ChoicesResponse> {
    let response = run_workflow_with_tasks(input.to_do_list, input.daily_schedule)
        .await
        .expect("error calling dify api:");

    let json = extract_json_response(&response.data)
        .expect("Failed to extract JSON response");

    let life_tasks = json["lifeTasks"].as_object()
        .expect("lifeTasks is not an object");
    let work_tasks = json["workTasks"].as_object()
        .expect("workTasks is not an object");

    let mut life_tasks_map = HashMap::new();
    let mut work_tasks_map = HashMap::new();

    // Extract life tasks
    for i in 1..=1 {
        let key = format!("lifeTask{}", i);
        let task = life_tasks.get(&key)
            .and_then(Value::as_str)
            .unwrap_or("(LLM generated empty task)")
            .to_string();
        life_tasks_map.insert(key, task);
    }

    // Extract work tasks
    for i in 1..=2 {
        let key = format!("workTask{}", i);
        let task = work_tasks.get(&key)
            .and_then(Value::as_str)
            .unwrap_or("(LLM generated empty task)")
            .to_string();
        work_tasks_map.insert(key, task);
    }

    Json(ChoicesResponse {
        life_tasks: life_tasks_map,
        work_tasks: work_tasks_map,
    })
}

async fn run_workflow_with_tasks(to_do_list: String, daily_schedule: String) -> Result<WorkflowsRunResponse> {
    let api_key = env::var("DIFY_API_KEY").expect("DIFY_API_KEY must be set in the .env file");
    let config = Config {
        base_url: "https://api.dify.ai".into(),
        api_key,
        timeout: Duration::from_secs(60),
    };
    let client = Client::new_with_config(config);

    let mut input_map = HashMap::new();
    input_map.insert("to_do_list".to_string(), to_do_list);
    input_map.insert("daily_schedule".to_string(), daily_schedule);

    let data = WorkflowsRunRequest {
        inputs: input_map,
        response_mode: request::ResponseMode::Blocking,
        user: "Moof".into(),
        files: Vec::new(),
    };

    let result = client.api().workflows_run(data).await?;
    Ok(result)
}

async fn hello_world() -> Markup {
    /*
    let result = question().await.unwrap();

    let debug_output = debug_workflow_result(&result.data);
    println!("Debug Information:\n{}", debug_output);
    /*
    let output: String = match result.data.outputs {
        Some(outputs) => {
            // Assuming the output is a string. If it's more complex, you'll need to handle it accordingly.
            outputs.get("result").and_then(|v| v.as_str()).unwrap_or("No output available").to_string()
        },
        None => "No output available".to_string(),
    }; */
    let output: String = match &result.data.outputs {
        Some(outputs) => {
            outputs.get("json response")
                .and_then(|v| v.as_str())
                .and_then(|json_str| serde_json::from_str::<Value>(json_str).ok())
                .map(|json| json.to_string())
                .unwrap_or_else(|| "Failed to parse JSON response".to_string())
        },
        None => "No outputs available".to_string(),
    };
    */
    html! {
        (DOCTYPE)
        html lang="en" data-theme="light" {
            head {
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                meta name="description" content="A bare-bones HTMX application";
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css";
                script src="https://unpkg.com/htmx.org@2.0.3" integrity="sha384-0895/pl2MU10Hqc6jd4RvrthNlDiE9U1tWmX7WRESftEDRosgxNsQG/Ze9YMRzHq" crossorigin="anonymous" {}
                title {"CS Club"}
            }
            body {
                main class="container" {
                    h1 { "CS Club" }
                    p {
                        "What's one of the best colleges for learning to program at an affordable price?"
                    }
                    section {
                        article {
                            // p { (output) }
                            p { "meow hey"}
                        }
                    }
                }
            }
        }
    }
}
// api end choices that gets a post request, I parse, pass it to dify, send it back to front end.

#[derive(Deserialize, Serialize)]
struct EchoRequest {
    message: String,
}

#[derive(Serialize)]
struct EchoResponse {
    message: String,
}

async fn echo(Json(payload): Json<EchoRequest>) -> Json<EchoResponse> {
    Json(EchoResponse {
        message: payload.message,
    })
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let schedule: String = "9:00 AM - Team meeting\n11:00 AM - Coding session\n2:00 PM - Lunch break\n3:00 PM - Final testing".to_string();
    let todo_list: String = "1. Finish project presentation\n2. Debug code\n3. Prepare demo".to_string();
    let result = run_workflow_with_tasks(todo_list, schedule).await.expect("blah");
    println!("{}", debug_workflow_result(&result.data));

    let api_key = env::var("DIFY_API_KEY").expect("error getting DIFY_API_KEY ");
    // println!("api key is {}", api_key);
    // build our application with a single route
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/echo", post(echo))
        .route("/choices", post(choices));

    // run it with hyper on localhost:7878
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    println!("Listening on 0.0.0.0:7878");
    axum::serve(listener, app.into_make_service()).await.unwrap();
}