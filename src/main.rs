use std::path::Path;

use crate::database::{Database, importer::ImportFormat};

mod database;

fn main() {
    simple_logger::init().expect("couldnt init logger");
    // let graph = Graph::<i64, i64>::new();
    // println!("Graph created: {graph:?}");

    let workspace_root = String::from("/home/mattys/skrypty-i-syfy/studia/inzynierka/raptorDB/");
    // let binding = workspace_root + "maps/sacz_mniejszy.osm";
    let binding = workspace_root.clone() + "maps/sacz_mniejszy.osm.pbf";
    println!("reading from {binding}");

    let map_path = Path::new(binding.as_str());
    let mut database = Database::new();

    database.import_graph(map_path, ImportFormat::PBF).expect("cant import graph");
    database.export_graph(Path::new((workspace_root + "export/sacz_mniejszy.geojson").as_str())).expect("cant export");
}
