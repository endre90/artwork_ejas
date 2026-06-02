# import json
# import numpy as np
# import matplotlib.pyplot as plt
# from datetime import datetime

# def load_part_data(filepath):
#     """Reads the aggregated JSON file and extracts dates and all three scores."""
#     with open(filepath, 'r') as f:
#         data = json.load(f)
    
#     dates = []
#     scores_manual = []
#     scores_initial = []
#     scores_rolling = []
    
#     for item in data:
#         day_data = item.get("day", {})
        
#         # Extract date info
#         date_info = day_data.get("date", {})
#         year = date_info.get("year", 2026)
#         month = date_info.get("month", 1)
#         day = date_info.get("day", 1)
#         dates.append(datetime(year, month, day))
        
#         # Extract scores
#         scores_manual.append(day_data.get("manual", 0))
#         scores_initial.append(day_data.get("initial", 0))
#         scores_rolling.append(day_data.get("rolling", 0))
        
#     return dates, scores_manual, scores_initial, scores_rolling

# def plot_subplot(ax, dates, scores_manual, scores_initial, scores_rolling, title):
#     """Applies the custom styling and plots data on a specific subplot (ax)."""
#     colors = {
#         "manual": "#e17674",   # Red
#         "initial": "#fdbb84",  # Orange
#         "rolling": "#a1d99b"   # Green
#     }

#     x_indices = np.arange(len(dates))

#     # Plot the lines
#     ax.plot(x_indices, scores_manual, color=colors["manual"], linewidth=2.5, marker='o', markersize=5, zorder=3, label="Manual")
#     ax.plot(x_indices, scores_initial, color=colors["initial"], linewidth=2.5, marker='o', markersize=5, zorder=3, label="Initial (Open-Loop)")
#     ax.plot(x_indices, scores_rolling, color=colors["rolling"], linewidth=2.5, marker='o', markersize=5, zorder=3, label="Rolling (Closed-Loop)")

#     # Add small numbers by each dot
#     for scores_list, color in [(scores_manual, colors["manual"]), 
#                                (scores_initial, colors["initial"]), 
#                                (scores_rolling, colors["rolling"])]:
#         for x, y in zip(x_indices, scores_list):
#             ax.annotate(f"{y:.0f}", 
#                         (x, y), 
#                         textcoords="offset points", 
#                         xytext=(0, 8), 
#                         ha='center', 
#                         fontsize=7,
#                         color=color,
#                         zorder=4)

#     # Clean up the axes by removing the outer box
#     for spine in ax.spines.values():
#         spine.set_visible(False)

#     # Add a thin, visible grid and put it behind the lines
#     ax.grid(color='#cccccc', linestyle='-', linewidth=0.5, zorder=1)
#     ax.set_axisbelow(True)
#     ax.tick_params(which="both", bottom=False, left=False)

#     # Format X-axis
#     tick_spacing = max(1, len(dates) // 10)
#     ticks_to_show = x_indices[::tick_spacing]
#     labels_to_show = [dates[i].strftime("%d.%m.") for i in ticks_to_show]
    
#     ax.set_xticks(ticks_to_show)
    
#     # --- INCREASED TICK SIZES HERE ---
#     ax.set_xticklabels(labels_to_show, fontsize=12, rotation=45)
#     ax.tick_params(axis='y', labelsize=12)
    
#     ax.set_title(title, fontsize=14, pad=15)

# def plot_all_parts(files, output_filename="four_part_evaluation.pdf"):
#     # Create a 2x2 grid of subplots
#     fig, axs = plt.subplots(2, 2, figsize=(18, 10))
#     axs = axs.flatten() # Flatten the 2x2 array to easily iterate 0 to 3

#     titles = ["16.01. - 23.01.", "02.02. - 06.02.", "11.02. - 17.02.", "27.02. - 10.03."]

#     for i, filepath in enumerate(files):
#         # Load the data for this specific part
#         dates, manual, initial, rolling = load_part_data(filepath)
        
#         # Plot it on the corresponding subplot
#         plot_subplot(axs[i], dates, manual, initial, rolling, titles[i])
        
#         # Add Y-label only to the leftmost charts to avoid visual clutter
#         if i % 2 == 0:
#             axs[i].set_ylabel("Daily Quality Score (DQS)", labelpad=10, fontsize=12)

#     # Extract handles and labels from the first subplot to create a single, unified legend
#     handles, labels = axs[0].get_legend_handles_labels()
    
#     # --- ADDED MAIN TITLE HERE ---
#     fig.suptitle("Volvo CE Assignment Quality Evaluation", fontsize=26)
    
#     # Lowered the legend slightly to make room for the new suptitle
#     fig.legend(handles, labels, loc='upper center', ncol=3, frameon=False, fontsize=13, bbox_to_anchor=(0.5, 0.94))

#     # Adjust layout so X-axis labels and titles don't overlap. 
#     # Using rect limits how high the subplots can draw, leaving room for title + legend.
#     plt.tight_layout(rect=[0, 0, 1, 0.90])
#     plt.subplots_adjust(hspace=0.45) 
    
#     plt.savefig(output_filename, bbox_inches="tight")
#     plt.show()

# if __name__ == "__main__":
#     # Define the paths to your 4 aggregated JSON files
#     file_paths = [
#         "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/VCE/VCE_part_a_combined_scores.json",
#         "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/VCE/VCE_part_b_combined_scores.json",
#         "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/VCE/VCE_part_c_combined_scores.json",
#         "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/VCE/VCE_part_d_combined_scores.json"
#     ]
    
#     plot_all_parts(file_paths)

import json
import numpy as np
import matplotlib.pyplot as plt
from datetime import datetime

def load_part_data(filepath):
    """Reads the aggregated JSON file and extracts dates and all three scores."""
    with open(filepath, 'r') as f:
        data = json.load(f)
    
    dates = []
    scores_manual = []
    scores_initial = []
    scores_rolling = []
    
    for item in data:
        day_data = item.get("day", {})
        
        # Extract date info
        date_info = day_data.get("date", {})
        year = date_info.get("year", 2026)
        month = date_info.get("month", 1)
        day = date_info.get("day", 1)
        dates.append(datetime(year, month, day))
        
        # Extract scores
        scores_manual.append(day_data.get("manual", 0))
        scores_initial.append(day_data.get("initial", 0))
        scores_rolling.append(day_data.get("rolling", 0))
        
    return dates, scores_manual, scores_initial, scores_rolling

def plot_subplot(ax, dates, scores_manual, scores_initial, scores_rolling, title):
    """Applies the custom styling and plots data on a specific subplot (ax)."""
    colors = {
        "manual": "#e17674",   # Red
        "initial": "#fdbb84",  # Orange
        "rolling": "#a1d99b"   # Green
    }

    x_indices = np.arange(len(dates))

    # --- INCREASED LINE WIDTH AND MARKER SIZE ---
    line_w = 5.5
    marker_s = 9
    
    # Plot the lines
    ax.plot(x_indices, scores_manual, color=colors["manual"], linewidth=line_w, marker='o', markersize=marker_s, zorder=3, label="Manual")
    ax.plot(x_indices, scores_initial, color=colors["initial"], linewidth=line_w, marker='o', markersize=marker_s, zorder=3, label="Initial (Open-Loop)")
    ax.plot(x_indices, scores_rolling, color=colors["rolling"], linewidth=line_w, marker='o', markersize=marker_s, zorder=3, label="Rolling (Closed-Loop)")

    # Add numbers by each dot
    # for scores_list, color in [(scores_manual, colors["manual"]), 
    #                            (scores_initial, colors["initial"]), 
    #                            (scores_rolling, colors["rolling"])]:
    #     for x, y in zip(x_indices, scores_list):
    #         # INCREASED ANNOTATION FONT SIZE AND PUSHED IT HIGHER (xytext) TO CLEAR THICK LINES
    #         ax.annotate(f"{y:.0f}", 
    #                     (x, y), 
    #                     textcoords="offset points", 
    #                     xytext=(0, 12), 
    #                     ha='center', 
    #                     fontsize=12,
    #                     fontweight='bold',
    #                     color=color,
    #                     zorder=4)

    # Clean up the axes by removing the outer box
    for spine in ax.spines.values():
        spine.set_visible(False)

    # Add a thin, visible grid and put it behind the lines
    ax.grid(color='#cccccc', linestyle='-', linewidth=0.5, zorder=1)
    ax.set_axisbelow(True)
    ax.tick_params(which="both", bottom=False, left=False)

    # Format X-axis
    tick_spacing = max(1, len(dates) // 10)
    ticks_to_show = x_indices[::tick_spacing]
    labels_to_show = [dates[i].strftime("%d.%m.") for i in ticks_to_show]
    
    ax.set_xticks(ticks_to_show)
    
    # --- INCREASED TICK SIZES ---
    ax.set_xticklabels(labels_to_show, fontsize=16, rotation=45)
    ax.tick_params(axis='y', labelsize=16)
    
    # --- INCREASED SUBTITLE SIZE ---
    ax.set_title(title, fontsize=22, pad=18, color="#333333")

def plot_all_parts(files, output_filename="four_part_evaluation.pdf"):
    # Create a 2x2 grid of subplots (increased height slightly to accommodate larger text)
    fig, axs = plt.subplots(2, 2, figsize=(20, 12))
    axs = axs.flatten() 

    titles = ["16.01. - 23.01.", "02.02. - 06.02.", "11.02. - 17.02.", "27.02. - 10.03."]

    for i, filepath in enumerate(files):
        # Load the data for this specific part
        dates, manual, initial, rolling = load_part_data(filepath)
        
        # Plot it on the corresponding subplot
        plot_subplot(axs[i], dates, manual, initial, rolling, titles[i])
        
        # --- INCREASED Y-LABEL SIZE ---
        if i % 2 == 0:
            axs[i].set_ylabel("Assignment Quality Score", labelpad=15, fontsize=18)

    # Extract handles and labels from the first subplot to create a single, unified legend
    handles, labels = axs[0].get_legend_handles_labels()
    
    # --- INCREASED MAIN TITLE SIZE ---
    fig.suptitle("Volvo CE Assignment Quality Evaluation", fontsize=32, y=0.98)
    
    # --- INCREASED LEGEND SIZE ---
    fig.legend(handles, labels, loc='upper center', ncol=3, frameon=False, fontsize=18, bbox_to_anchor=(0.5, 0.95))

    # Adjust layout so X-axis labels and titles don't overlap. 
    plt.tight_layout(rect=[0, 0, 1, 0.93])
    # --- INCREASED HSPACE TO PREVENT LARGE X-TICKS FROM HITTING THE BOTTOM OF THE TOP CHARTS ---
    plt.subplots_adjust(hspace=0.55) 
    
    plt.savefig(output_filename, bbox_inches="tight")
    plt.show()

if __name__ == "__main__":
    # Define the paths to your 4 aggregated JSON files
    file_paths = [
        "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/VCE/VCE_part_a_combined_scores.json",
        "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/VCE/VCE_part_b_combined_scores.json",
        "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/VCE/VCE_part_c_combined_scores.json",
        "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/VCE/VCE_part_d_combined_scores.json"
    ]
    
    plot_all_parts(file_paths)