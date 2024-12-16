use burn::data::dataset::Dataset;
use gtno::data::MD17Dataset;
use gtno::model::IMPGTNO;

fn main() {
    let dataset = MD17Dataset::new().unwrap();
    let item = dataset.get(4).unwrap();
    println!("{}", item);
}
