# import json
# import matplotlib.pyplot as plt
# import numpy as np
# from matplotlib.colors import ListedColormap

# # Predefined operations per station
# STATION_OPS = {
#     "S1": ["O1","O2","O3","O4","O5","O6","O7"],
#     "S2": ["O1","O2","O3","O4","O5","O6","O7","O8","O9"],
#     "S3": ["O1","O2","O3","O4","O5","O6"],
#     "S4": ["O1","O2","O3","O4","O5","O6","O7","O8"]
# }

# def build_competence_matrix(people, operations):
#     # Create a matrix: rows = operations, cols = operators
#     # 1 if operator is competent in operation, else 0
#     operators = [p["name"] for p in people]
#     op_idx = {op: i for i, op in enumerate(operations)}
#     operator_idx = {o: i for i, o in enumerate(operators)}

#     mat = np.zeros((len(operations), len(operators)), dtype=int)
#     for p_i, person in enumerate(people):
#         for c in person["competences"]:
#             if c in op_idx:
#                 mat[op_idx[c], p_i] = 1
#     return mat, operators

# def plot_heatmap(station_name, operations, people):
#     mat, operators = build_competence_matrix(people, operations)

#     fig, ax = plt.subplots(figsize=(10, 6))
#     fig.suptitle(f"Station {station_name}: Competence Matrix", fontsize=14)

#     # Define a simple colormap: white (0) and green (1)
#     cmap = ListedColormap(["white", "green"])

#     im = ax.imshow(mat, aspect='auto', cmap=cmap, interpolation='nearest')
    
#     # Set tick labels
#     ax.set_xticks(np.arange(len(operators)))
#     ax.set_yticks(np.arange(len(operations)))
#     ax.set_xticklabels(operators, rotation=45, ha='right', fontsize=8)
#     ax.set_yticklabels(operations, fontsize=8)

#     ax.set_xlabel("Operators")
#     ax.set_ylabel("Operations")

#     # Create a colorbar
#     cbar = plt.colorbar(im, ax=ax, fraction=0.046, pad=0.04)
#     cbar.set_ticks([0,1])
#     cbar.set_ticklabels(["No Competence","Competent"])

#     plt.tight_layout(rect=[0, 0.03, 1, 0.95])
#     plt.show()

# def main():
#     with open("/home/endre/rust_crates/artwork_ejas/data/matrix.json", "r") as f:
#         data = json.load(f)

#     stations = data["stations"]

#     # Plot a heatmap for each station
#     for station_name in sorted(stations.keys()):
#         # If the station_name is known, use predefined ops. 
#         # Otherwise, derive from data if needed.
#         operations = STATION_OPS.get(station_name, None)
#         if operations is None:
#             # If operations not predefined, gather from people competences
#             ops_set = set()
#             for p in stations[station_name]["people"]:
#                 for c in p["competences"]:
#                     ops_set.add(c)
#             operations = sorted(ops_set)
        
#         people = stations[station_name]["people"]
#         plot_heatmap(station_name, operations, people)

# if __name__ == "__main__":
#     main()


# import json
# import numpy as np
# import matplotlib.pyplot as plt
# from matplotlib.colors import LinearSegmentedColormap

# # Predefined operations per station
# STATION_OPS = {
#     "S1": ["O1","O2","O3","O4","O5","O6","O7"],
#     "S2": ["O1","O2","O3","O4","O5","O6","O7","O8","O9"],
#     "S3": ["O1","O2","O3","O4","O5","O6"],
#     "S4": ["O1","O2","O3","O4","O5","O6","O7","O8"]
# }

# def build_matrix(people, operations):
#     """
#     Build a matrix of preference scores:
#     - np.nan for no competence (white)
#     - 0.0 for competence but no preference (green)
#     - (0,1] for preferences, where 1 = top preference (purple),
#       and values decrease as preferences get lower.
      
#     Also returns a matrix of labels:
#     - "" for no competence
#     - "–" for competence no preference
#     - a rank number as a string for preferences (e.g. "1" for top preference)
#     """
#     operators = [p["name"] for p in people]
#     mat = np.full((len(operations), len(operators)), np.nan)
#     labels = [["" for _ in operators] for _ in operations]

#     # Extract preference info per person
#     # Each person's preferences are a list. 
#     # If n = number of preferences, the top preference (index i=0) gets score=1, 
#     # next gets score slightly less, etc.
#     prefs_dict = {}
#     for person in people:
#         n = len(person["preferences"])
#         pref_scores = {}
#         if n > 0:
#             for i, op in enumerate(person["preferences"]):
#                 score = 1 - i/n  # top pref i=0 => score=1, last pref => score close to >0
#                 pref_scores[op] = (score, i+1)
#         # Store preferences mapping for this person
#         prefs_dict[person["name"]] = pref_scores

#     op_idx = {op: i for i, op in enumerate(operations)}
#     for col, person in enumerate(people):
#         pname = person["name"]
#         for c in person["competences"]:
#             if c in op_idx:
#                 row = op_idx[c]
#                 if c in prefs_dict[pname]:
#                     # This operation is in preferences
#                     score, rank = prefs_dict[pname][c]
#                     mat[row, col] = score  # (0,1]
#                     labels[row][col] = str(rank)
#                 else:
#                     # Competent but not preferred
#                     mat[row, col] = 0.0
#                     labels[row][col] = "–"
#     return mat, labels, operators

# def plot_station_heatmap(station_name, station_data):
#     operations = STATION_OPS.get(station_name, None)
#     if operations is None:
#         # If not predefined, derive from station_data
#         ops_set = set()
#         for p in station_data["people"]:
#             for c in p["competences"]:
#                 ops_set.add(c)
#         operations = sorted(ops_set)

#     people = station_data["people"]
#     mat, labels, operators = build_matrix(people, operations)

#     fig, ax = plt.subplots(figsize=(10,6))
#     fig.suptitle(f"Station {station_name}: Competence & Preference Heatmap", fontsize=14)

#     # Create a colormap from green (no preference) to purple (top preference)
#     # green = (0,1,0), purple = (0.5,0,0.5)
#     colors = [(0,1,0), (0.5,0,0.5)]
#     cmap = LinearSegmentedColormap.from_list("GreenPurple", colors)
    
#     # Show image
#     im = ax.imshow(mat, aspect='auto', cmap=cmap, interpolation='nearest')
#     im.set_clim(0,1)  # data in [0,1]
#     im.set_bad('white')  # nan -> white (no competence)

#     # Ticks and labels
#     ax.set_xticks(np.arange(len(operators)))
#     ax.set_yticks(np.arange(len(operations)))
#     ax.set_xticklabels(operators, rotation=45, ha='right', fontsize=8)
#     ax.set_yticklabels(operations, fontsize=8)

#     ax.set_xlabel("Operators")
#     ax.set_ylabel("Operations")

#     # Draw faint dotted grid lines between cells
#     ax.set_xticks(np.arange(-0.5, len(operators), 1), minor=True)
#     ax.set_yticks(np.arange(-0.5, len(operations), 1), minor=True)
#     ax.grid(which='minor', color='black', linestyle=':', linewidth=0.5, alpha=0.3)
#     ax.tick_params(which="minor", bottom=False, left=False)

#     # Overlay the labels
#     # Text color depends on background intensity for readability
#     for i in range(len(operations)):
#         for j in range(len(operators)):
#             text = labels[i][j]
#             if text:
#                 val = mat[i,j]
#                 if np.isnan(val):
#                     # White background -> black text
#                     color = 'black'
#                 else:
#                     # If val > 0.5 (closer to purple), white text for contrast
#                     color = 'white' if val > 0.5 else 'black'
#                 ax.text(j, i, text, ha='center', va='center', color=color, fontsize=8)

#     # Create a colorbar
#     # 0 = no pref (green), 1 = top pref (purple)
#     cbar = plt.colorbar(im, ax=ax, fraction=0.046, pad=0.04)
#     cbar.set_ticks([0,1])
#     cbar.set_ticklabels(["No/Low Pref","Top Pref"])

#     plt.tight_layout(rect=[0, 0.03, 1, 0.95])
#     plt.show()

# def main():
#     with open("/home/endre/rust_crates/artwork_ejas/data/matrix.json", "r") as f:
#         data = json.load(f)

#     stations = data["stations"]

#     for station_name in sorted(stations.keys()):
#         plot_station_heatmap(station_name, stations[station_name])

# if __name__ == "__main__":
#     main()


import json
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap

# Predefined operations per station
STATION_OPS = {
    "S1": ["O1","O2","O3","O4","O5","O6","O7"],
    "S2": ["O1","O2","O3","O4","O5","O6","O7","O8","O9"],
    "S3": ["O1","O2","O3","O4","O5","O6"],
    "S4": ["O1","O2","O3","O4","O5","O6","O7","O8"]
}

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

    # Create a quick lookup of preferences per person
    # We'll store them in a dict: {person_name: {op: rank_score}}
    # rank_score = 1 - i/n where i is index in preferences
    prefs_dict = {}
    for person in people:
        n = len(person["preferences"])
        pref_scores = {}
        for i, op in enumerate(person["preferences"]):
            if n > 0:
                score = 1 - i/n
            else:
                score = 0
            pref_scores[op] = (score, i+1)  # also store rank (i+1)
        prefs_dict[person["name"]] = pref_scores

    op_idx = {op: i for i, op in enumerate(operations)}
    for col, person in enumerate(people):
        pname = person["name"]
        for op in person["competences"]:
            if op in op_idx:
                row = op_idx[op]
                # Competent
                if op in prefs_dict[pname]:
                    # Has preference
                    score, rank = prefs_dict[pname][op]
                    mat[row, col] = score  # between (0,1]
                    labels[row][col] = str(rank)
                else:
                    # Competent but no preference
                    mat[row, col] = 0.0
                    labels[row][col] = "–"
    return mat, labels, operators

def plot_station_heatmap(station_name, station_data):
    operations = STATION_OPS.get(station_name, None)
    if operations is None:
        # If not predefined, derive from station_data
        ops_set = set()
        for p in station_data["people"]:
            for c in p["competences"]:
                ops_set.add(c)
        operations = sorted(ops_set)

    people = station_data["people"]
    mat, labels, operators = build_matrix(people, operations)

    # fig, ax = plt.subplots(figsize=(10,6))
    fig, ax = plt.subplots(figsize=(6, 4))
    # fig.suptitle(f"Station {station_name}: Competence & Preference Heatmap", fontsize=14)

    # Create a colormap from green to purple
    # green = (0,1,0), purple = (0.5,0,0.5)
    colors = [(0,0.5,0), (0.5,0,0.5)]
    cmap = LinearSegmentedColormap.from_list("GreenPurple", colors)
    
    # Set NaN values to appear as white
    im = ax.imshow(mat, aspect='auto', cmap=cmap, interpolation='nearest')
    im.set_clim(0,1)  # data in [0,1]
    im.set_gid('white')  # nan -> white

    # Ticks and labels
    ax.set_xticks(np.arange(len(operators)))
    ax.set_yticks(np.arange(len(operations)))
    ax.set_xticklabels(operators, rotation=45, ha='right', fontsize=8)
    ax.set_yticklabels(operations, fontsize=8)

    ax.set_xlabel("Operators")
    ax.set_ylabel("Operations")

    # Draw faint dotted grid lines between cells
    # We can use minor ticks and grid
    ax.set_xticks(np.arange(-0.5, len(operators), 1), minor=True)
    ax.set_yticks(np.arange(-0.5, len(operations), 1), minor=True)
    ax.grid(which='minor', color='black', linestyle=':', linewidth=0.5, alpha=0.3)
    ax.tick_params(which="minor", bottom=False, left=False)

    # Overlay the labels
    for i in range(len(operations)):
        for j in range(len(operators)):
            text = labels[i][j]
            if text:
                # If there's a preference score or "–", show it
                # Use white or black text depending on background
                val = mat[i,j]
                if np.isnan(val):
                    # no competence, cell is white, use black text
                    color = 'black'
                else:
                    # if value > 0.5 (more purple), use white text for contrast
                    # else use black
                    if val > 0.5:
                        color = 'white'
                    else:
                        color = 'white'
                ax.text(j, i, text, ha='center', va='center', color=color, fontsize=10)

    # Create a colorbar
    # We'll manually make a colorbar that shows the gradient from green to purple
    # and labels: "No Pref" near green and "Top Pref" near purple
    cbar = plt.colorbar(im, ax=ax, fraction=0.046, pad=0.04)
    cbar.set_ticks([0,1])
    cbar.set_ticklabels(["No/Low","High"])

    plt.tight_layout(rect=[0, 0.03, 1, 0.95])
    plt.show()

def main():
    with open("/home/endre/rust_crates/artwork_ejas/data/matrix.json", "r") as f:
        data = json.load(f)

    stations = data["stations"]

    for station_name in sorted(stations.keys()):
        plot_station_heatmap(station_name, stations[station_name])

if __name__ == "__main__":
    main()
