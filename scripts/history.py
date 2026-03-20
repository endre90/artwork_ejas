# import json
# import numpy as np
# import matplotlib.pyplot as plt
# from matplotlib.colors import ListedColormap
# import matplotlib.patches as mpatches
# from datetime import date, timedelta
# from matplotlib.patches import Patch

# def process_data(data):
#     # Sort data chronologically
#     data_sorted = sorted(data, key=lambda x: (x["day"]["date"]["year"], x["day"]["date"]["month"], x["day"]["date"]["day"]))
    
#     # Extract unique operators
#     operators = sorted(list(set(op[0] for d in data_sorted for op in d["day"]["assignments"])))
    
#     # Parse dates and map them directly to their assignments
#     date_to_assignments = {}
#     for d in data_sorted:
#         dt = date(d["day"]["date"]["year"], d["day"]["date"]["month"], d["day"]["date"]["day"])
#         date_to_assignments[dt] = d["day"]["assignments"]
        
#     dates_in_data = set(date_to_assignments.keys())
    
#     if not dates_in_data:
#         return np.array([]), np.array([]), [], []
        
#     min_date = min(dates_in_data)
#     max_date = max(dates_in_data)
    
#     row_definitions = []
#     gap_start = None
#     gap_end = None

#     def flush_gap():
#         """Helper to compress consecutive missing days into a single row."""
#         if gap_start is not None:
#             if gap_start == gap_end:
#                 label = f"{gap_start.day:02d}.{gap_start.month:02d}."
#             elif gap_start.month == gap_end.month:
#                 label = f"{gap_start.day:02d}-{gap_end.day:02d}.{gap_start.month:02d}."
#             else:
#                 label = f"{gap_start.day:02d}.{gap_start.month:02d}.-{gap_end.day:02d}.{gap_end.month:02d}."
            
#             row_definitions.append({"type": "gap", "label": label})

#     # Iterate through the full timeline to group gaps and insert known days
#     curr_date = min_date
#     while curr_date <= max_date:
#         if curr_date in dates_in_data:
#             flush_gap()
#             gap_start = None
#             gap_end = None
            
#             # Add the active working day
#             row_definitions.append({
#                 "type": "known", 
#                 "date": curr_date, 
#                 "label": f"{curr_date.day:02d}.{curr_date.month:02d}."
#             })
#         else:
#             if curr_date.weekday() < 5:  # It's a missing weekday (Mon-Fri)
#                 if gap_start is None:
#                     gap_start = curr_date
#                 gap_end = curr_date
#         curr_date += timedelta(days=1)
        
#     flush_gap() # Catch any trailing gaps
    
#     # Initialize matrices: Y-axis (Rows) = Timeline, X-axis (Cols) = Operators
#     # Fill with "U" and our new Unknown integer (4) by default
#     text_mat = np.full((len(row_definitions), len(operators)), "U", dtype=object)
#     int_mat = np.full((len(row_definitions), len(operators)), 4, dtype=int) 
    
#     op_to_idx = {op: i for i, op in enumerate(operators)}
    
#     # Populate the matrices
#     for row_idx, row_def in enumerate(row_definitions):
#         if row_def["type"] == "known":
#             assignments = date_to_assignments[row_def["date"]]
            
#             for assignment in assignments:
#                 operator, operation = assignment[0], assignment[1]
#                 col_idx = op_to_idx[operator]
#                 text_mat[row_idx, col_idx] = operation
                
#                 # Categorize the operation for coloring
#                 if operation.startswith("O"):
#                     int_mat[row_idx, col_idx] = 0  # Green (Operations)
#                 elif operation == "T":
#                     int_mat[row_idx, col_idx] = 1  # Purple (Training)
#                 elif operation == "L":
#                     int_mat[row_idx, col_idx] = 2  # Red (Leave)
#                 elif operation in ["E", "S"]:
#                     int_mat[row_idx, col_idx] = 3  # Gray (Extra/Other)
#                 else:
#                     int_mat[row_idx, col_idx] = 3  # Fallback to gray

#     row_labels = [r["label"] for r in row_definitions]
#     return text_mat, int_mat, row_labels, operators

# def plot_assignment_history(text_mat, int_mat, days, operators):
#     # Dynamic sizing so it perfectly fits however many compressed rows we generate
#     fig, ax = plt.subplots(figsize=(len(operators) * 0.75, len(days) * 0.3))

#     # Reusing the established color palette + White for "Unknown"
#     colors = [
#         "#a1d99b", 
#         "#fdbb84", 
#         "#9e9ac8", 
#         "#e17674", 
#         "#ffffff", 
#     ]
#     cmap = ListedColormap(colors)

#     # Plot the matrix
#     im = ax.imshow(int_mat, aspect='auto', cmap=cmap, interpolation='nearest')
#     im.set_clim(-0.5, 4.5)

#     # Configure Axes (Removed fontweight='bold' for labels and ticks)
#     ax.set_xticks(np.arange(len(operators)))
#     ax.set_yticks(np.arange(len(days)))
#     ax.set_xticklabels(operators, fontsize=11)
#     ax.set_yticklabels(days, fontsize=11)

#     ax.set_xlabel("Operators", labelpad=10, fontsize=13)
#     ax.set_ylabel("Date Timeline", labelpad=10, fontsize=13)

#     #     ax.set_xticklabels(operators, fontsize=11)
#     # ax.set_yticklabels(date_labels, fontsize=11)

#     # ax.set_xlabel("Operators", labelpad=15, fontsize=13)
#     # ax.set_ylabel("Dates", labelpad=15, fontsize=13)

#     # Remove outer black box (spines)
#     for spine in ax.spines.values():
#         spine.set_visible(False)

#     # Draw solid white lines between cells (tile effect)
#     ax.set_xticks(np.arange(-0.5, len(operators), 1), minor=True)
#     ax.set_yticks(np.arange(-0.5, len(days), 1), minor=True)
#     ax.grid(which='minor', color='white', linestyle='-', linewidth=2)
#     ax.tick_params(which="both", bottom=False, left=False)

#     # Overlay the text labels and hatching (Restored fontweight='bold')
#     for i in range(len(days)):         # rows
#         for j in range(len(operators)): # cols
#             text = text_mat[i, j]
            
#             if text == "U":
#                 # Draw a cross-hatched rectangle over the Unknown cells
#                 rect = plt.Rectangle((j - 0.5, i - 0.5), 1, 1, fill=False, hatch='////', edgecolor='#a6a6a6', lw=0)
#                 ax.add_patch(rect)
#                 # Lighter text for the U so it doesn't overpower the actual data
#                 ax.text(j, i, text, ha='center', va='center', color='#a6a6a6', fontsize=10, fontweight='bold')
#             elif text:
#                 # Normal dark text for active assignments
#                 ax.text(j, i, text, ha='center', va='center', color='#333333', fontsize=10, fontweight='bold')

#     # # Create Custom Legend
#     # legend_labels = ["Operation", "Training (T)", "Loaned out (L)", "Absent (E)"]


#     # legend_patches = [mpatches.Patch(facecolor=colors[i], edgecolor='white', label=legend_labels[i]) for i in range(4)]
    
#     # # Add the special hatched patch for Unknown
#     # legend_patches.append(mpatches.Patch(facecolor='#ffffff', edgecolor='#a6a6a6', hatch='////', label='Unknown (U)'))

#     legend_elements = [
#         Patch(facecolor='#aec7e8', edgecolor='white', label='TL (Team Leader)'),
#         Patch(facecolor='#a1d99b', edgecolor='white', label='OX (Operation)'),
#         Patch(facecolor='#e17674', edgecolor='white', label='E (Absent)'),
#         Patch(facecolor='#fdbb84', edgecolor='white', label='T (Training)'),
#         Patch(facecolor='#9e9ac8', edgecolor='white', label='L (Loaned out)'),
#         Patch(facecolor='#ffffff', edgecolor='#a6a6a6', hatch='////', label='U (Unknown)')
#     ]
    
#     # Legend placed safely underneath the title
#     ax.legend(handles=legend_elements, loc='lower center', bbox_to_anchor=(0.5, 1.02), ncol=7, frameon=False, fontsize=10)
    
#     # ax.legend(handles=legend_patches, loc='lower center', bbox_to_anchor=(0.5, 1.02), ncol=5, frameon=False, fontsize=10)

#     # Add the title text below the figure (Bumped up the y-coordinate to 0.04 to bring it closer)
#     # ax.set_title("VCE Station: Assignment History", fontsize=16, fontweight='bold', pad=45)
#     ax.set_title("Volvo CE Station Daily Assignments in January, February, and March 2026", fontsize=16, pad=40)
    
#     # Adjusted rect to tighten the gap between the axes and the text
#     plt.tight_layout(rect=[0, 0.06, 1, 1])
#     plt.savefig("history_vce.pdf", bbox_inches="tight")
#     plt.show()

# def main():
#     file_path = "/home/endre/rust_ws/artwork_ejas/data/factory/VCE_history.json" 
    
#     with open(file_path, "r") as f:
#         raw_data = json.load(f)
    
#     # Process and plot
#     text_mat, int_mat, days, operators = process_data(raw_data)
#     plot_assignment_history(text_mat, int_mat, days, operators)

# if __name__ == "__main__":
#     main()

import json
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import ListedColormap
import matplotlib.patches as mpatches
from datetime import date, timedelta
from matplotlib.patches import Patch

def process_data(data):
    # Sort data chronologically
    data_sorted = sorted(data, key=lambda x: (x["day"]["date"]["year"], x["day"]["date"]["month"], x["day"]["date"]["day"]))
    
    # Extract unique operators
    operators = sorted(list(set(op[0] for d in data_sorted for op in d["day"]["assignments"])))
    
    # Parse dates and map them directly to their assignments AND leader
    date_to_day_info = {}
    for d in data_sorted:
        dt = date(d["day"]["date"]["year"], d["day"]["date"]["month"], d["day"]["date"]["day"])
        date_to_day_info[dt] = {
            "assignments": d["day"]["assignments"],
            "leader": d["day"].get("leader")  # Extract the leader field
        }
        
    dates_in_data = set(date_to_day_info.keys())
    
    if not dates_in_data:
        return np.array([]), np.array([]), [], []
        
    min_date = min(dates_in_data)
    max_date = max(dates_in_data)
    
    row_definitions = []
    gap_start = None
    gap_end = None

    def flush_gap():
        """Helper to compress consecutive missing days into a single row."""
        if gap_start is not None:
            if gap_start == gap_end:
                label = f"{gap_start.day:02d}.{gap_start.month:02d}."
            elif gap_start.month == gap_end.month:
                label = f"{gap_start.day:02d}-{gap_end.day:02d}.{gap_start.month:02d}."
            else:
                label = f"{gap_start.day:02d}.{gap_start.month:02d}.-{gap_end.day:02d}.{gap_end.month:02d}."
            
            row_definitions.append({"type": "gap", "label": label})

    # Iterate through the full timeline to group gaps and insert known days
    curr_date = min_date
    while curr_date <= max_date:
        if curr_date in dates_in_data:
            flush_gap()
            gap_start = None
            gap_end = None
            
            # Add the active working day
            row_definitions.append({
                "type": "known", 
                "date": curr_date, 
                "label": f"{curr_date.day:02d}.{curr_date.month:02d}."
            })
        else:
            if curr_date.weekday() < 5:  # It's a missing weekday (Mon-Fri)
                if gap_start is None:
                    gap_start = curr_date
                gap_end = curr_date
        curr_date += timedelta(days=1)
        
    flush_gap() # Catch any trailing gaps
    
    # Initialize matrices: Y-axis (Rows) = Timeline, X-axis (Cols) = Operators
    # Fill with "U" and our new Unknown integer (4) by default
    text_mat = np.full((len(row_definitions), len(operators)), "U", dtype=object)
    int_mat = np.full((len(row_definitions), len(operators)), 4, dtype=int) 
    
    op_to_idx = {op: i for i, op in enumerate(operators)}
    
    # Populate the matrices
    for row_idx, row_def in enumerate(row_definitions):
        if row_def["type"] == "known":
            day_info = date_to_day_info[row_def["date"]]
            assignments = day_info["assignments"]
            leader = day_info["leader"]
            
            for assignment in assignments:
                operator, operation = assignment[0], assignment[1]
                col_idx = op_to_idx[operator]
                text_mat[row_idx, col_idx] = operation
                
                # Categorize the operation for coloring
                if operator == leader:
                    int_mat[row_idx, col_idx] = 5  # Blue (Team Leader) overrides other states
                elif operation.startswith("O"):
                    int_mat[row_idx, col_idx] = 0  # Green (Operations)
                elif operation == "T":
                    int_mat[row_idx, col_idx] = 1  # Training
                elif operation == "L":
                    int_mat[row_idx, col_idx] = 2  # Loaned out
                elif operation in ["E", "S"]:
                    int_mat[row_idx, col_idx] = 3  # Absent
                else:
                    int_mat[row_idx, col_idx] = 3  # Fallback

    row_labels = [r["label"] for r in row_definitions]
    return text_mat, int_mat, row_labels, operators

def plot_assignment_history(text_mat, int_mat, days, operators):
    # Dynamic sizing so it perfectly fits however many compressed rows we generate
    fig, ax = plt.subplots(figsize=(len(operators) * 0.75, len(days) * 0.3))

    # Color palette
    colors = [
        "#a1d99b", # 0: Operation (Soft Green)
        "#fdbb84", # 1: Training (Orange)
        "#9e9ac8", # 2: Loaned Out (Purple)
        "#e17674", # 3: Absent (Red)
        "#ffffff", # 4: Unknown (White)
        "#aec7e8", # 5: Team Leader (Pastel Blue)
    ]
    cmap = ListedColormap(colors)

    # Plot the matrix
    im = ax.imshow(int_mat, aspect='auto', cmap=cmap, interpolation='nearest')
    im.set_clim(-0.5, 5.5) # Increased upper limit to 5.5 to include state 5

    # Configure Axes
    ax.set_xticks(np.arange(len(operators)))
    ax.set_yticks(np.arange(len(days)))
    ax.set_xticklabels(operators, fontsize=11)
    ax.set_yticklabels(days, fontsize=11)

    ax.set_xlabel("Operators", labelpad=10, fontsize=13)
    ax.set_ylabel("Dates", labelpad=10, fontsize=13)

    # Remove outer black box (spines)
    for spine in ax.spines.values():
        spine.set_visible(False)

    # Draw solid white lines between cells (tile effect)
    ax.set_xticks(np.arange(-0.5, len(operators), 1), minor=True)
    ax.set_yticks(np.arange(-0.5, len(days), 1), minor=True)
    ax.grid(which='minor', color='white', linestyle='-', linewidth=2)
    ax.tick_params(which="both", bottom=False, left=False)

    # Overlay the text labels and hatching
    for i in range(len(days)):         # rows
        for j in range(len(operators)): # cols
            text = text_mat[i, j]
            
            if text == "U":
                # Draw a cross-hatched rectangle over the Unknown cells
                rect = plt.Rectangle((j - 0.5, i - 0.5), 1, 1, fill=False, hatch='////', edgecolor='#a6a6a6', lw=0)
                ax.add_patch(rect)
                # Lighter text for the U so it doesn't overpower the actual data
                ax.text(j, i, text, ha='center', va='center', color='#a6a6a6', fontsize=10, fontweight='bold')
            elif text:
                # Normal dark text for active assignments
                ax.text(j, i, text, ha='center', va='center', color='#333333', fontsize=10, fontweight='bold')

    legend_elements = [
        Patch(facecolor='#aec7e8', edgecolor='white', label='TL (Team Leader)'),
        Patch(facecolor='#a1d99b', edgecolor='white', label='OX (Operation)'),
        Patch(facecolor='#e17674', edgecolor='white', label='E (Absent)'),
        Patch(facecolor='#fdbb84', edgecolor='white', label='T (Training)'),
        Patch(facecolor='#9e9ac8', edgecolor='white', label='L (Loaned out)'),
        Patch(facecolor='#ffffff', edgecolor='#a6a6a6', hatch='////', label='U (Unknown)')
    ]
    
    # Legend placed safely underneath the title
    ax.legend(handles=legend_elements, loc='lower center', bbox_to_anchor=(0.5, 1.02), ncol=7, frameon=False, fontsize=10)

    # Add the title text
    ax.set_title("Volvo CE Station Daily Assignments in January, February, and March 2026", fontsize=16, pad=40)
    
    # Adjusted rect to tighten the gap between the axes and the text
    plt.tight_layout(rect=[0, 0.06, 1, 1])
    plt.savefig("history_vce.pdf", bbox_inches="tight")
    plt.show()

def main():
    file_path = "/home/endre/rust_ws/artwork_ejas/data/factory/VCE_history.json" 
    
    with open(file_path, "r") as f:
        raw_data = json.load(f)
    
    # Process and plot
    text_mat, int_mat, days, operators = process_data(raw_data)
    plot_assignment_history(text_mat, int_mat, days, operators)

if __name__ == "__main__":
    main()