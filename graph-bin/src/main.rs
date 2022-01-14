use std::{
    fs::File,
    io::BufReader,
    env,
};
use anyhow::{
    Context,
    Result,
};
use graph_lib::Graph;

fn main() {
    let file_name = env::args().nth(1).expect("filename with \"Trivial Graph Format\" not set");
    graph_processing(&file_name).unwrap();
}

fn graph_processing(file_name: &str) -> Result<()> {
    let file = File::open(file_name)
        .with_context(|| format!("error reading file \"{}\"", file_name))?;

    let graph = Graph::deserialize(BufReader::new(file))?;
    let bfs_result = graph.bfs_random_start()?;
    for (id, value, neighbours) in bfs_result {
        if let Some(value) = value {
            println!("{} {} {:?}", id, value, neighbours);
        }
        else {
            println!("{} {:?}", id, neighbours);
        }
    };

    Ok(())
}
