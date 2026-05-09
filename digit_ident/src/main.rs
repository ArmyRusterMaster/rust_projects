mod nn;
mod data_loader;

use std::env;

fn main() {
    // Initialize minimal network and data loader
    let mut net = nn::MLP::new(784, 128, 10);
    let _loader = data_loader::DataLoader::new();

    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "train" => {
            println!("Training scaffold...");
            let x = vec![0.0f32; 784];
            let _out = net.forward(&x);
        }
        "eval" => {
            println!("Evaluation scaffold...");
        }
        "infer" => {
            println!("Inference scaffold...");
            let x = vec![0.0f32; 784];
            let _out = net.forward(&x);
        }
        _ => {
            println!("Usage: {} [train|eval|infer]", args.get(0).unwrap_or(&"mnist_mlp".to_string()));
        }
    }
}
