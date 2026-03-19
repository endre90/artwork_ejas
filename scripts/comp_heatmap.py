import json
import re
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap

# Predefined operations per station
STATION_OPS = {
    "GTO": ["O1","O2","O3","O4","O5","O6","O7","O8"],
    "CE": ["O1", "O2", "O3", "O4", "O5", "O6", "O7", "O8", "O9", "O10", "O11", "O12"],
    "S1": ["O1","O2","O3","O4","O5","O6","O7"],
    "S2": ["O1","O2","O3","O4","O5","O6","O7","O8","O9"],
    "S3": ["O1","O2","O3","O4","O5","O6"],
    "S4": ["O1","O2","O3","O4","O5","O6","O7","O8"]
}

def natural_sort_key(s):
    """Splits strings into letters and numbers for natural sorting (O1, O2... O9, O10)"""
    return [int(text) if text.isdigit() else text.lower() for text in re.split(r'(\d+)', s)]

def build_matrix(people, operations):
    """
    Build a matrix of preference scores:
    - np.nan for no competence
    - 0.0 for competence but no preference
    - (0,1] for preferences, depending on rank
    Also returns a matrix of labels to put inside the cells:
    - "" for no competence
    - "–" for competence no preference
    - rank number (as string) for preferences
    """
    operators = [p["name"] for p in people]
    mat = np.full((len(operations), len(operators)), np.nan)
    labels = [["" for _ in operators] for _ in operations]

    prefs_dict = {}
    for person in people:
        n = len(person["preferences"])
        pref_scores = {}
        for i, op in enumerate(person["preferences"]):
            if n > 0:
                score = 1 - i/n
            else:
                score = 0
            pref_scores[op] = (score, i+1)
        prefs_dict[person["name"]] = pref_scores

    op_idx = {op: i for i, op in enumerate(operations)}
    for col, person in enumerate(people):
        pname = person["name"]
        for op in person["competences"]:
            if op in op_idx:
                row = op_idx[op]
                if op in prefs_dict[pname]:
                    score, rank = prefs_dict[pname][op]
                    mat[row, col] = score
                    labels[row][col] = str(rank)
                else:
                    mat[row, col] = 0.0
                    labels[row][col] = "–"
    return mat, labels, operators

def plot_station_heatmap(station_name, station_data):
    operations = STATION_OPS.get(station_name, None)
    if operations is None:
        ops_set = set()
        for p in station_data["people"]:
            for c in p["competences"]:
                ops_set.add(c)
        # Use natural sorting here so O10 comes after O9
        operations = sorted(ops_set, key=natural_sort_key)

    people = station_data["people"]
    mat, labels, operators = build_matrix(people, operations)

    fig, ax = plt.subplots(figsize=(8, 5))

    colors = ["#a1d99b", "#3f007d"]
    cmap = LinearSegmentedColormap.from_list("SoftGreenPurple", colors)
    
    # The middle-ground red for no competence
    cmap.set_bad(color='#e17674')

    im = ax.imshow(mat, aspect='auto', cmap=cmap, interpolation='nearest')
    im.set_clim(0,1)

    # Ticks and labels
    ax.set_xticks(np.arange(len(operators)))
    ax.set_yticks(np.arange(len(operations)))
    ax.set_xticklabels(operators, ha='center', fontsize=9)
    ax.set_yticklabels(operations, fontsize=9)

    ax.set_xlabel("Operators", labelpad=10, fontsize=11)
    ax.set_ylabel("Operations", labelpad=10, fontsize=11)

    # Remove outer black box (spines)
    for spine in ax.spines.values():
        spine.set_visible(False)

    # Draw solid white lines between cells
    ax.set_xticks(np.arange(-0.5, len(operators), 1), minor=True)
    ax.set_yticks(np.arange(-0.5, len(operations), 1), minor=True)
    ax.grid(which='minor', color='white', linestyle='-', linewidth=2)
    ax.tick_params(which="both", bottom=False, left=False)

    # Overlay the labels
    for i in range(len(operations)):
        for j in range(len(operators)):
            val = mat[i,j]
            if np.isnan(val):
                # White X for the tuned-down red background
                ax.text(j, i, "X", ha='center', va='center', color='white', fontsize=10, fontweight='bold')
            else:
                text = labels[i][j]
                if text:
                    color = 'white' if val > 0.4 else 'black'
                    ax.text(j, i, text, ha='center', va='center', color=color, fontsize=10, fontweight='bold')

    # Cleaner colorbar
    cbar = plt.colorbar(im, ax=ax, fraction=0.046, pad=0.04)
    cbar.outline.set_visible(False)
    cbar.set_ticks([0, 1])
    cbar.set_ticklabels(["No/Low Pref", "Top Pref"])

    plt.title(f"Volvo {station_name} station", fontsize=15, pad=15)
    plt.tight_layout()
    plt.show()

def main():
    with open("/home/endre/rust_ws/artwork_ejas/data/factory/VCE_matrix.json", "r") as f:
        data = json.load(f)

    stations = data["stations"]

    for station_name in sorted(stations.keys()):
        plot_station_heatmap(station_name, stations[station_name])

if __name__ == "__main__":
    main()