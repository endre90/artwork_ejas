# # import json
# # import numpy as np
# # import matplotlib.pyplot as plt
# # from matplotlib.colors import LinearSegmentedColormap

# # # Predefined operations per station
# # STATION_OPS = {
# #     "GTO": ["O1","O2","O3","O4","O5","O6","O7","O8"],
# #     "S1": ["O1","O2","O3","O4","O5","O6","O7"],
# #     "S2": ["O1","O2","O3","O4","O5","O6","O7","O8","O9"],
# #     "S3": ["O1","O2","O3","O4","O5","O6"],
# #     "S4": ["O1","O2","O3","O4","O5","O6","O7","O8"]
# # }

# # def build_matrix(people, operations):
# #     """
# #     Build a matrix of preference scores:
# #     - np.nan for no competence
# #     - 0.0 for competence but no preference
# #     - (0,1] for preferences, depending on rank
# #     Also returns a matrix of labels to put inside the cells:
# #     - "" for no competence
# #     - "–" for competence no preference
# #     - rank number (as string) for preferences
# #     """
# #     operators = [p["name"] for p in people]
# #     mat = np.full((len(operations), len(operators)), np.nan)
# #     labels = [["" for _ in operators] for _ in operations]

# #     # Create a quick lookup of preferences per person
# #     # We'll store them in a dict: {person_name: {op: rank_score}}
# #     # rank_score = 1 - i/n where i is index in preferences
# #     prefs_dict = {}
# #     for person in people:
# #         n = len(person["preferences"])
# #         pref_scores = {}
# #         for i, op in enumerate(person["preferences"]):
# #             if n > 0:
# #                 score = 1 - i/n
# #             else:
# #                 score = 0
# #             pref_scores[op] = (score, i+1)  # also store rank (i+1)
# #         prefs_dict[person["name"]] = pref_scores

# #     op_idx = {op: i for i, op in enumerate(operations)}
# #     for col, person in enumerate(people):
# #         pname = person["name"]
# #         for op in person["competences"]:
# #             if op in op_idx:
# #                 row = op_idx[op]
# #                 # Competent
# #                 if op in prefs_dict[pname]:
# #                     # Has preference
# #                     score, rank = prefs_dict[pname][op]
# #                     mat[row, col] = score  # between (0,1]
# #                     labels[row][col] = str(rank)
# #                 else:
# #                     # Competent but no preference
# #                     mat[row, col] = 0.0
# #                     labels[row][col] = "–"
# #     return mat, labels, operators

# # def plot_station_heatmap(station_name, station_data):
# #     operations = STATION_OPS.get(station_name, None)
# #     if operations is None:
# #         # If not predefined, derive from station_data
# #         ops_set = set()
# #         for p in station_data["people"]:
# #             for c in p["competences"]:
# #                 ops_set.add(c)
# #         operations = sorted(ops_set)

# #     people = station_data["people"]
# #     mat, labels, operators = build_matrix(people, operations)

# #     # fig, ax = plt.subplots(figsize=(10,6))
# #     fig, ax = plt.subplots(figsize=(6, 4))
# #     # fig.suptitle(f"Station {station_name}: Competence & Preference Heatmap", fontsize=14)

# #     # Create a colormap from green to purple
# #     # green = (0,1,0), purple = (0.5,0,0.5)
# #     colors = [(0,0.5,0), (0.5,0,0.5)]
# #     cmap = LinearSegmentedColormap.from_list("GreenPurple", colors)
    
# #     # Set NaN values to appear as white
# #     im = ax.imshow(mat, aspect='auto', cmap=cmap, interpolation='nearest')
# #     im.set_clim(0,1)  # data in [0,1]
# #     im.set_gid('white')  # nan -> white

# #     # Ticks and labels
# #     ax.set_xticks(np.arange(len(operators)))
# #     ax.set_yticks(np.arange(len(operations)))
# #     ax.set_xticklabels(operators, rotation=45, ha='right', fontsize=8)
# #     ax.set_yticklabels(operations, fontsize=8)

# #     ax.set_xlabel("Operators")
# #     ax.set_ylabel("Operations")

# #     # Draw faint dotted grid lines between cells
# #     # We can use minor ticks and grid
# #     ax.set_xticks(np.arange(-0.5, len(operators), 1), minor=True)
# #     ax.set_yticks(np.arange(-0.5, len(operations), 1), minor=True)
# #     ax.grid(which='minor', color='black', linestyle=':', linewidth=0.5, alpha=0.3)
# #     ax.tick_params(which="minor", bottom=False, left=False)

# #     # Overlay the labels
# #     for i in range(len(operations)):
# #         for j in range(len(operators)):
# #             text = labels[i][j]
# #             if text:
# #                 # If there's a preference score or "–", show it
# #                 # Use white or black text depending on background
# #                 val = mat[i,j]
# #                 if np.isnan(val):
# #                     # no competence, cell is white, use black text
# #                     color = 'black'
# #                 else:
# #                     # if value > 0.5 (more purple), use white text for contrast
# #                     # else use black
# #                     if val > 0.5:
# #                         color = 'white'
# #                     else:
# #                         color = 'white'
# #                 ax.text(j, i, text, ha='center', va='center', color=color, fontsize=10)

# #     # Create a colorbar
# #     # We'll manually make a colorbar that shows the gradient from green to purple
# #     # and labels: "No Pref" near green and "Top Pref" near purple
# #     cbar = plt.colorbar(im, ax=ax, fraction=0.046, pad=0.04)
# #     cbar.set_ticks([0,1])
# #     cbar.set_ticklabels(["No/Low","High"])

# #     plt.tight_layout(rect=[0, 0.03, 1, 0.95])
# #     plt.show()

# # def main():
# #     with open("/home/endre/rust_ws/artwork_ejas/data/factory/GTO_matrix.json", "r") as f:
# #         data = json.load(f)

# #     stations = data["stations"]

# #     for station_name in sorted(stations.keys()):
# #         plot_station_heatmap(station_name, stations[station_name])

# # if __name__ == "__main__":
# #     main()

# import re
# import json
# import numpy as np
# import matplotlib.pyplot as plt
# from matplotlib.colors import LinearSegmentedColormap

# def natural_sort_key(s):
#     """Splits strings into letters and numbers for natural sorting (O1, O2... O9, O10)"""
#     return [int(text) if text.isdigit() else text.lower() for text in re.split(r'(\d+)', s)]

# # Predefined operations per station
# STATION_OPS = {
#     "GTO": ["O1","O2","O3","O4","O5","O6","O7","O8"],
#     "S1": ["O1","O2","O3","O4","O5","O6","O7"],
#     "S2": ["O1","O2","O3","O4","O5","O6","O7","O8","O9"],
#     "S3": ["O1","O2","O3","O4","O5","O6"],
#     "S4": ["O1","O2","O3","O4","O5","O6","O7","O8"]
# }

# def build_matrix(people, operations):
#     """
#     Build a matrix of preference scores:
#     - np.nan for no competence
#     - 0.0 for competence but no preference
#     - (0,1] for preferences, depending on rank
#     Also returns a matrix of labels to put inside the cells:
#     - "" for no competence
#     - "–" for competence no preference
#     - rank number (as string) for preferences
#     """
#     operators = [p["name"] for p in people]
#     mat = np.full((len(operations), len(operators)), np.nan)
#     labels = [["" for _ in operators] for _ in operations]

#     # Create a quick lookup of preferences per person
#     prefs_dict = {}
#     for person in people:
#         n = len(person["preferences"])
#         pref_scores = {}
#         for i, op in enumerate(person["preferences"]):
#             if n > 0:
#                 score = 1 - i/n
#             else:
#                 score = 0
#             pref_scores[op] = (score, i+1)  # also store rank (i+1)
#         prefs_dict[person["name"]] = pref_scores

#     op_idx = {op: i for i, op in enumerate(operations)}
#     for col, person in enumerate(people):
#         pname = person["name"]
#         for op in person["competences"]:
#             if op in op_idx:
#                 row = op_idx[op]
#                 # Competent
#                 if op in prefs_dict[pname]:
#                     # Has preference
#                     score, rank = prefs_dict[pname][op]
#                     mat[row, col] = score  # between (0,1]
#                     labels[row][col] = str(rank)
#                 else:
#                     # Competent but no preference
#                     mat[row, col] = 0.0
#                     labels[row][col] = "–"
#     return mat, labels, operators

# def plot_station_heatmap(station_name, station_data):
#     operations = STATION_OPS.get(station_name, None)
#     if operations is None:
#         ops_set = set()
#         for p in station_data["people"]:
#             for c in p["competences"]:
#                 ops_set.add(c)
#         # operations = sorted(ops_set)
#         sorted(ops_set, key=natural_sort_key)

#     people = station_data["people"]
#     mat, labels, operators = build_matrix(people, operations)

#     # Slightly adjusted figure size for better proportions
#     fig, ax = plt.subplots(figsize=(8, 5))

#     # Prettier, softer colormap (light green to deep purple)
#     # colors = ["#c7e9c0", "#3f007d"]
#     # cmap = LinearSegmentedColormap.from_list("SoftGreenPurple", colors)
    
#     # # Correct way to handle NaN colors in matplotlib
#     # cmap.set_bad(color='#f5f5f5')

#     colors = ["#a1d99b", "#3f007d"]
#     cmap = LinearSegmentedColormap.from_list("SoftGreenPurple", colors)
    
#     # Set no competence (NaN values) to a nice visible red
#     # cmap.set_bad(color='#d9534f')
#     cmap.set_bad(color='#e17674')
#     # e89999

#     im = ax.imshow(mat, aspect='auto', cmap=cmap, interpolation='nearest')
#     im.set_clim(0,1)

#     # Ticks and labels
#     ax.set_xticks(np.arange(len(operators)))
#     ax.set_yticks(np.arange(len(operations)))
#     # ax.set_xticklabels(operators, rotation=45, ha='right', fontsize=9)
#     ax.set_xticklabels(operators, ha='center', fontsize=9)
#     ax.set_yticklabels(operations, fontsize=9)

#     ax.set_xlabel("Operators", fontweight='bold', labelpad=10)
#     ax.set_ylabel("Operations", fontweight='bold', labelpad=10)

#     # Remove outer black box (spines) for a modern look
#     for spine in ax.spines.values():
#         spine.set_visible(False)

#     # Draw solid white lines between cells to create a "tile" effect
#     ax.set_xticks(np.arange(-0.5, len(operators), 1), minor=True)
#     ax.set_yticks(np.arange(-0.5, len(operations), 1), minor=True)
#     ax.grid(which='minor', color='white', linestyle='-', linewidth=2)
    
#     # Remove all tick marks (keep the labels, drop the little lines)
#     ax.tick_params(which="both", bottom=False, left=False)

#     # Overlay the labels with correct contrast logic
#     for i in range(len(operations)):
#         for j in range(len(operators)):
#             text = labels[i][j]
#             if text:
#                 val = mat[i,j]
#                 if np.isnan(val):
#                     color = 'black'
#                 else:
#                     # Fixed contrast: dark text on light cells, white text on dark cells
#                     color = 'white' if val > 0.4 else 'black'
#                 ax.text(j, i, text, ha='center', va='center', color=color, fontsize=10, fontweight='bold')

#     # Cleaner colorbar
#     cbar = plt.colorbar(im, ax=ax, fraction=0.046, pad=0.04)
#     cbar.outline.set_visible(False) # Remove colorbar border
#     cbar.set_ticks([0, 1])
#     cbar.set_ticklabels(["No/Low Pref", "Top Pref"])

#     plt.title(f"Station {station_name}", fontsize=14, fontweight='bold', pad=15)
#     plt.tight_layout()
#     plt.show()

# def main():
#     with open("/home/endre/rust_ws/artwork_ejas/data/factory/VCE_matrix.json", "r") as f:
#         data = json.load(f)

#     stations = data["stations"]

#     for station_name in sorted(stations.keys()):
#         plot_station_heatmap(station_name, stations[station_name])

# if __name__ == "__main__":
#     main()


import json
import re
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap

# Predefined operations per station
STATION_OPS = {
    "GTO": ["O1","O2","O3","O4","O5","O6","O7","O8"],
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

    ax.set_xlabel("Operators", fontweight='bold', labelpad=10)
    ax.set_ylabel("Operations", fontweight='bold', labelpad=10)

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

    plt.title(f"Station {station_name}", fontsize=14, fontweight='bold', pad=15)
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