use burn::data::dataset::{Dataset, InMemDataset};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GraphAtTimestep {
    #[serde(rename = "timestep")]
    pub timestep: i32,
    #[serde(rename = "energy")]
    pub energy: f32,
    pub nuclear_charges: Vec<i32>, // Stacked charges
    pub coords: Vec<[f32; 3]>,     // Stacked coordinates
    pub forces: Vec<[f32; 3]>,     // Stacked forces
}

pub struct MD17Dataset {
    dataset: InMemDataset<GraphAtTimestep>,
}

impl MD17Dataset {
    pub fn new() -> Result<Self, std::io::Error> {
        let path = PathBuf::from("../data/rmd17_cleaned/rmd17_aspirin.csv");
        let file = std::fs::File::open(&path)?;
        let mut reader = csv::Reader::from_reader(file);

        // Extract headers to identify fields for charges, coords, and forces
        let headers = reader.headers()?;
        let mut charge_indices = Vec::new();
        let mut coord_indices = Vec::new();
        let mut force_indices = Vec::new();

        for (i, header) in headers.iter().enumerate() {
            if header.ends_with("_charge") {
                charge_indices.push(i);
            } else if header.ends_with("_coord") {
                coord_indices.push(i);
            } else if header.ends_with("_force") {
                force_indices.push(i);
            }
        }

        println!("Charge indices: {:?}", charge_indices);
        println!("Coord indices: {:?}", coord_indices);
        println!("Force indices: {:?}", force_indices);

        // Manually build dataset
        let mut dataset_vec = Vec::new();
        for result in reader.records() {
            let record = result?;

            // Parse general fields
            let timestep: i32 = record.get(0).unwrap().parse().unwrap();
            let energy: f32 = record.get(1).unwrap().parse().unwrap();

            // Parse charges into Vec<i32>
            let mut nuclear_charges = Vec::new();
            for &idx in &charge_indices {
                let charge: i32 = record.get(idx).unwrap().parse().unwrap();
                nuclear_charges.push(charge);
            }

            // Parse coordinates into Vec<[f32; 3]>
            let mut coords = Vec::new();
            for &idx in &coord_indices {
                if let Some(value) = record.get(idx) {
                    match parse_vector_field(value) {
                        Ok(coord) => coords.push(coord),
                        Err(e) => eprintln!("Error parsing coordinate at index {}: {}", idx, e),
                    }
                }
            }

            // Parse forces into Vec<[f32; 3]>
            let mut forces = Vec::new();
            for &idx in &force_indices {
                if let Some(value) = record.get(idx) {
                    match parse_vector_field(value) {
                        Ok(force) => forces.push(force),
                        Err(e) => eprintln!("Error parsing force at index {}: {}", idx, e),
                    }
                }
            }

            // Add GraphAtTimestep to dataset
            dataset_vec.push(GraphAtTimestep {
                timestep,
                energy,
                nuclear_charges,
                coords,
                forces,
            });
        }

        // Create an InMemDataset
        let dataset = InMemDataset::new(dataset_vec);

        Ok(Self { dataset })
    }
}

impl Dataset<GraphAtTimestep> for MD17Dataset {
    fn get(&self, index: usize) -> Option<GraphAtTimestep> {
        self.dataset.get(index)
    }

    fn len(&self) -> usize {
        self.dataset.len()
    }
}

impl fmt::Display for GraphAtTimestep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Graph at Timestep:\n\
             --------------------\n\
             Timestep: {}\n\
             Energy: {:.6}\n\
             Nuclear Charges:\n{:?}\n\
             Coordinates:\n{}\n\
             Forces:\n{}\n",
            self.timestep,
            self.energy,
            self.nuclear_charges,
            self.coords
                .iter()
                .map(|coord| format!("  [{:.6}, {:.6}, {:.6}]", coord[0], coord[1], coord[2]))
                .collect::<Vec<String>>()
                .join("\n"),
            self.forces
                .iter()
                .map(|force| format!("  [{:.6}, {:.6}, {:.6}]", force[0], force[1], force[2]))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

fn parse_vector_field(value: &str) -> Result<[f32; 3], String> {
    // Remove brackets and split by commas
    let value = value.trim_matches(|c| c == '[' || c == ']');
    let parsed_values: Result<Vec<f32>, _> = value.split(',').map(|v| v.trim().parse()).collect();

    match parsed_values {
        Ok(values) if values.len() == 3 => Ok([values[0], values[1], values[2]]),
        Ok(_) => Err(format!("Field does not have 3 components: {:?}", value)),
        Err(e) => Err(format!("Error parsing field {:?}: {:?}", value, e)),
    }
}
