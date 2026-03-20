# import json
# import numpy as np
# import matplotlib.pyplot as plt
# from matplotlib.colors import ListedColormap
# from matplotlib.patches import Patch
# from datetime import date

# def process_schedule_data(data):
#     """
#     Extracts dates and operators, building a 2x2 sub-grid for passes.
#     Rows (Y-axis): Dates (2 sub-rows per date for passes 1/2 and 3/4)
#     Cols (X-axis): Operators (2 sub-cols per operator for passes 1/3 and 2/4)
#     """
#     date_set = set()
#     operator_set = set()
    
#     for entry in data:
#         p = entry.get("pass", {})
#         dt = date(p["date"]["year"], p["date"]["month"], p["date"]["day"])
#         date_set.add(dt)
#         for person, op in p["assignments"]:
#             operator_set.add(person)
            
#     dates = sorted(list(date_set))
#     operators = sorted(list(operator_set))
    
#     # Create the high-res matrices (2x size for the 2x2 sub-grids)
#     mat_rows = len(dates) * 2
#     mat_cols = len(operators) * 2
    
#     int_mat = np.full((mat_rows, mat_cols), np.nan)
#     label_mat = [["" for _ in range(mat_cols)] for _ in range(mat_rows)]
    
#     def get_status_int(op_code):
#         if op_code == "E": return 1
#         if op_code == "S": return 2
#         if op_code == "L": return 3
#         if op_code == "T": return 4
#         if op_code == "N": return 5 # Added mapping for No Production
#         if op_code == "TL": return 6
#         return 0 # 0 is the default for all Operations (O1, O2, etc.)

#     for entry in data:
#         p = entry.get("pass", {})
#         dt = date(p["date"]["year"], p["date"]["month"], p["date"]["day"])
#         pass_num = p["pass"] # Expecting 1, 2, 3, or 4
        
#         d_i = dates.index(dt)
        
#         for person, op in p["assignments"]:
#             op_i = operators.index(person)
            
#             # Map passes to the 2x2 quadrant inside the Date/Operator intersection
#             # P1: Top-Left, P2: Top-Right, P3: Bottom-Left, P4: Bottom-Right
#             sub_r = 0 if pass_num in [1, 2] else 1
#             sub_c = 0 if pass_num in [1, 3] else 1
            
#             r = d_i * 2 + sub_r
#             c = op_i * 2 + sub_c
            
#             # Format to P1O3, P2E, etc. (No spaces or colons)
#             label_mat[r][c] = f"P{pass_num}{op}"
#             int_mat[r, c] = get_status_int(op)
            
#     date_labels = [dt.strftime('%b %d') for dt in dates]
#     return int_mat, label_mat, date_labels, operators

# def plot_schedule_grid(int_mat, label_mat, date_labels, operators):
#     # Dynamically size the figure so it doesn't get squished
#     fig, ax = plt.subplots(figsize=(len(operators) * 1.0 + 2.2, len(date_labels) * 0.4 + 2.2))

#     colors = ["#a1d99b", "#e17674", "#fdbb84", "#9e9ac8", "#fdbb84", "#ffffff", "#aec7e8"] 
#     cmap = ListedColormap(colors)
#     cmap.set_bad(color='#ffffff') # Blank spots remain white

#     im = ax.imshow(int_mat, aspect='auto', cmap=cmap, interpolation='nearest')
#     im.set_clim(-0.5, 5.5) # Increased upper limit to 5.5 to include state 5

#     # --- Setup the Axes ---
#     # Place ticks exactly in the middle of each 2x2 block
#     ax.set_xticks(np.arange(0.5, len(operators) * 2, 2))
#     ax.set_yticks(np.arange(0.5, len(date_labels) * 2, 2))
    
#     ax.set_xticklabels(operators, fontsize=11)
#     ax.set_yticklabels(date_labels, fontsize=11)

#     ax.set_xlabel("Operators", labelpad=15, fontsize=13)
#     ax.set_ylabel("Dates", labelpad=15, fontsize=13)

#     # Remove default spines and ticks
#     for spine in ax.spines.values():
#         spine.set_visible(False)
#     ax.tick_params(which="both", bottom=False, left=False)

#     # --- Draw the Custom Grid ---
#     # Outer Lines: Thick white lines separating the GROUPS of 4 passes
#     for x in np.arange(1.5, len(operators) * 2 - 1, 2):
#         ax.axvline(x, color='white', linewidth=3, zorder=10)
#     for y in np.arange(1.5, len(date_labels) * 2 - 1, 2):
#         ax.axhline(y, color='white', linewidth=3, zorder=10)

#     # --- Overlay the Text and Hatching ---
#     for i in range(len(date_labels) * 2):
#         for j in range(len(operators) * 2):
#             text = label_mat[i][j]
#             if text:
#                 # Slice off the "P1" or "P2" to get just the operation code ("O3", "E", "N", etc.)
#                 op_code = text[2:] 
                
#                 if op_code == "N":
#                     # Draw a cross-hatched rectangle over the No Production cells
#                     # zorder=5 keeps it under the thick white gridlines but over the background
#                     rect = plt.Rectangle((j - 0.5, i - 0.5), 1, 1, fill=False, hatch='////', edgecolor='#a6a6a6', lw=0, zorder=5)
#                     ax.add_patch(rect)
#                     ax.text(j, i, "N", ha='center', va='center', color='#a6a6a6', fontsize=10, fontweight='bold', zorder=15)
#                 else:
#                     # Fade out 'E' text, keep everything else dark and bold
#                     # text_color = '#a0a0a0' if op_code == "E" else '#333333'
#                     text_color = '#333333'
#                     ax.text(j, i, text, ha='center', va='center', color=text_color, fontsize=10, fontweight='bold', zorder=15)

#     # --- Custom Legend ---
#     legend_elements = [
#         Patch(facecolor='#a1d99b', edgecolor='white', label='Active Operation'),
#         Patch(facecolor='#aec7e8', edgecolor='white', label='TL (Team Leader)'),
#         Patch(facecolor='#e17674', edgecolor='white', label='E (Absent)'),
#         Patch(facecolor='#fdbb84', edgecolor='white', label='S/T (Supervision or Training)'),
#         Patch(facecolor='#9e9ac8', edgecolor='white', label='L (Loaned out)'),
#         Patch(facecolor='#ffffff', edgecolor='#a6a6a6', hatch='////', label='N (No Prod)')
#     ]
    
#     # Legend placed safely underneath the title. Set ncol=6 for a single wide row.
#     ax.legend(handles=legend_elements, loc='lower center', bbox_to_anchor=(0.5, 1.02), ncol=7, frameon=False, fontsize=10)

#     # Adjusted padding slightly since the legend is now only one row tall
#     ax.set_title("Volvo GTO Station Daily Assignments (4 Passes / Day)", fontsize=16, pad=40)
    
#     plt.tight_layout()
#     plt.savefig("gto_history_nov.pdf", bbox_inches="tight")
#     plt.show()

# def main():
#     with open("/home/endre/rust_ws/artwork_ejas/data/factory/GTO_nov_history.json", "r") as f:
#         data = json.load(f)

#     int_mat, label_mat, date_labels, operators = process_schedule_data(data)
#     plot_schedule_grid(int_mat, label_mat, date_labels, operators)

# if __name__ == "__main__":
#     main()

import json
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import ListedColormap
from matplotlib.patches import Patch
from datetime import date

def process_schedule_data(data):
    """
    Extracts dates and operators, building a 2x2 sub-grid for passes.
    Rows (Y-axis): Dates (2 sub-rows per date for passes 1/2 and 3/4)
    Cols (X-axis): Operators (2 sub-cols per operator for passes 1/3 and 2/4)
    """
    date_set = set()
    operator_set = set()
    
    for entry in data:
        p = entry.get("pass", {})
        dt = date(p["date"]["year"], p["date"]["month"], p["date"]["day"])
        date_set.add(dt)
        for person, op in p["assignments"]:
            operator_set.add(person)
            
    dates = sorted(list(date_set))
    operators = sorted(list(operator_set))
    
    # Create the high-res matrices (2x size for the 2x2 sub-grids)
    mat_rows = len(dates) * 2
    mat_cols = len(operators) * 2
    
    int_mat = np.full((mat_rows, mat_cols), np.nan)
    label_mat = [["" for _ in range(mat_cols)] for _ in range(mat_rows)]
    
    def get_status_int(op_code):
        if op_code == "E": return 1
        if op_code == "S": return 2
        if op_code == "L": return 3
        if op_code == "T": return 4
        if op_code == "N": return 5 # Added mapping for No Production
        if op_code == "TL": return 6
        return 0 # 0 is the default for all Operations (O1, O2, etc.)

    for entry in data:
        p = entry.get("pass", {})
        dt = date(p["date"]["year"], p["date"]["month"], p["date"]["day"])
        pass_num = p["pass"] # Expecting 1, 2, 3, or 4
        
        d_i = dates.index(dt)
        
        for person, op in p["assignments"]:
            op_i = operators.index(person)
            
            # Map passes to the 2x2 quadrant inside the Date/Operator intersection
            # P1: Top-Left, P2: Top-Right, P3: Bottom-Left, P4: Bottom-Right
            sub_r = 0 if pass_num in [1, 2] else 1
            sub_c = 0 if pass_num in [1, 3] else 1
            
            r = d_i * 2 + sub_r
            c = op_i * 2 + sub_c
            
            # Format to P1O3, P2E, etc. (No spaces or colons)
            label_mat[r][c] = f"P{pass_num}{op}"
            int_mat[r, c] = get_status_int(op)
            
    # Fixed the date format to be DD.MM. (e.g., 17.11.)
    date_labels = [dt.strftime('%d.%m.') for dt in dates]
    return int_mat, label_mat, date_labels, operators

def plot_schedule_grid(int_mat, label_mat, date_labels, operators):
    # Dynamically size the figure so it doesn't get squished
    fig, ax = plt.subplots(figsize=(len(operators) * 1.0 + 2.2, len(date_labels) * 0.4 + 2.2))

    colors = ["#a1d99b", "#e17674", "#fdbb84", "#9e9ac8", "#fdbb84", "#ffffff", "#aec7e8"] 
    cmap = ListedColormap(colors)
    cmap.set_bad(color='#ffffff') # Blank spots remain white

    im = ax.imshow(int_mat, aspect='auto', cmap=cmap, interpolation='nearest')
    im.set_clim(-0.5, 6.5) # Increased upper limit to 6.5 to properly include state 6 (TL)

    # --- Setup the Axes ---
    # Place ticks exactly in the middle of each 2x2 block
    ax.set_xticks(np.arange(0.5, len(operators) * 2, 2))
    ax.set_yticks(np.arange(0.5, len(date_labels) * 2, 2))
    
    ax.set_xticklabels(operators, fontsize=11)
    ax.set_yticklabels(date_labels, fontsize=11)

    ax.set_xlabel("Operators", labelpad=15, fontsize=13)
    ax.set_ylabel("Dates", labelpad=15, fontsize=13)

    # Remove default spines and ticks
    for spine in ax.spines.values():
        spine.set_visible(False)
    ax.tick_params(which="both", bottom=False, left=False)

    # --- Draw the Custom Grid ---
    # Outer Lines: Thick white lines separating the GROUPS of 4 passes
    for x in np.arange(1.5, len(operators) * 2 - 1, 2):
        ax.axvline(x, color='white', linewidth=3, zorder=10)
    for y in np.arange(1.5, len(date_labels) * 2 - 1, 2):
        ax.axhline(y, color='white', linewidth=3, zorder=10)

    # --- Overlay the Text and Hatching ---
    for i in range(len(date_labels) * 2):
        for j in range(len(operators) * 2):
            text = label_mat[i][j]
            if text:
                # Slice off the "P1" or "P2" to get just the operation code ("O3", "E", "N", etc.)
                op_code = text[2:] 
                
                if op_code == "N":
                    # Draw a cross-hatched rectangle over the No Production cells
                    # zorder=5 keeps it under the thick white gridlines but over the background
                    rect = plt.Rectangle((j - 0.5, i - 0.5), 1, 1, fill=False, hatch='////', edgecolor='#a6a6a6', lw=0, zorder=5)
                    ax.add_patch(rect)
                    ax.text(j, i, "N", ha='center', va='center', color='#a6a6a6', fontsize=10, fontweight='bold', zorder=15)
                else:
                    # Keep everything dark and bold
                    text_color = '#333333'
                    ax.text(j, i, text, ha='center', va='center', color=text_color, fontsize=10, fontweight='bold', zorder=15)

    # --- Custom Legend ---
    legend_elements = [
        Patch(facecolor='#aec7e8', edgecolor='white', label='TL (Team Leader)'),
        Patch(facecolor='#a1d99b', edgecolor='white', label='OX (Operation)'),
        Patch(facecolor='#e17674', edgecolor='white', label='E (Absent)'),
        Patch(facecolor='#fdbb84', edgecolor='white', label='S/T (Supervision or Training)'),
        Patch(facecolor='#9e9ac8', edgecolor='white', label='L (Loaned out)'),
        Patch(facecolor='#ffffff', edgecolor='#a6a6a6', hatch='////', label='N (No Production)')
    ]
    
    # Legend placed safely underneath the title
    ax.legend(handles=legend_elements, loc='lower center', bbox_to_anchor=(0.5, 1.02), ncol=7, frameon=False, fontsize=10)

    # Adjusted padding slightly since the legend is now only one row tall
    ax.set_title("Volvo GTO Station Daily Assignments in January 2026 (4 Passes / Day)", fontsize=16, pad=40)
    
    plt.tight_layout()
    plt.savefig("gto_history_jan.pdf", bbox_inches="tight")
    plt.show()

def main():
    with open("/home/endre/rust_ws/artwork_ejas/data/factory/GTO_jan_history.json", "r") as f:
        data = json.load(f)

    int_mat, label_mat, date_labels, operators = process_schedule_data(data)
    plot_schedule_grid(int_mat, label_mat, date_labels, operators)

if __name__ == "__main__":
    main()