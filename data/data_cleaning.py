import numpy as np
import pandas as pd
from pathlib import Path


DROP_OLD_DATA = True
DROP_HYDROGEN = True


input_dir = Path("data/rmd17")
assert input_dir.exists(), "Input directory must exist"
assert input_dir.is_dir(), "Input directory must be a directory"
output_dir = Path("data/rmd17_cleaned")
output_dir.mkdir(exist_ok=True)

### Shape
# nuclear_charges: atom_count
# coords: timestep x atom_count x 3
# energy: timestep
# forces: timestep x atom_count x 3

for npz_file in input_dir.glob("*.npz"):
    print(npz_file)
    data = np.load(npz_file)
    if DROP_OLD_DATA:
        filtered_data: dict[str, np.ndarray] = {key: value for key, value in data.items() if not key.startswith("old_")}
    else:
        filtered_data = dict(data)  # Convert to a mutable dictionary

    if DROP_HYDROGEN:
        # Get indices of non-hydrogen atoms (nuclear charge != 1)
        non_h_indices = np.where(filtered_data["nuclear_charges"] != 1)[0]

        # Filter nuclear charges
        filtered_data["nuclear_charges"] = filtered_data["nuclear_charges"][non_h_indices]

        # Filter coordinates (keep all timesteps, filter atom dimension)
        filtered_data["coords"] = filtered_data["coords"][:, non_h_indices, :]

        # Filter forces (keep all timesteps, filter atom dimension)
        filtered_data["forces"] = filtered_data["forces"][:, non_h_indices, :]

    # Convert to DataFrame
    num_timesteps = filtered_data["energies"].shape[0]
    num_atoms = filtered_data["nuclear_charges"].shape[0]

    # Create a list to hold the rows of the DataFrame
    rows = []
    for t in range(num_timesteps):
        row = {"timestep": t, "energy": filtered_data["energies"][t]}
        for i in range(num_atoms):
            row[f"atom_{i}_charge"] = filtered_data["nuclear_charges"][i]
            row[f"atom_{i}_coord"] = filtered_data["coords"][t, i, :].tolist()
            row[f"atom_{i}_force"] = filtered_data["forces"][t, i, :].tolist()
        rows.append(row)

    df = pd.DataFrame(rows)

    # Save to CSV
    csv_filename = output_dir / f"{npz_file.stem}.csv"
    df.to_csv(csv_filename, index=False)

    print(f"Saved data from {npz_file.name} to {csv_filename}")

print(f"All .npz files have been processed and saved as .csv files in {output_dir}.")
