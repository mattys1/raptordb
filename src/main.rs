use std::path::Path;

use crate::{exporter::export_geojson, graph::NodeID, importer::{import_pbf, import_xml}};

mod graph;
mod importer;
mod exporter;

fn main() {
    simple_logger::init().expect("couldnt init logger");
    // let graph = Graph::<i64, i64>::new();
    // println!("Graph created: {graph:?}");

    let workspace_root = String::from("/home/mattys/skrypty-i-syfy/studia/inzynierka/raptorDB/");
    // let binding = workspace_root + "maps/sacz_mniejszy.osm";
    let binding = workspace_root.clone() + "maps/sacz_mniejszy.osm.pbf";
    println!("reading from {binding}");

    let map_path = Path::new(binding.as_str());
    let graph = import_pbf(map_path).expect("fuck");
    
    export_geojson(&graph, Path::new((workspace_root + "export/sacz_mniejszy.geojson").as_str())).expect("cant export");
}
