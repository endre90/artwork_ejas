import json
import numpy as np
import matplotlib.pyplot as plt
from datetime import datetime

def load_evaluation_data(filepath):
    """Reads the JSON file and extracts the dates and total scores. 
       Injects np.nan for passes marked as No Production ('N')."""
    with open(filepath, 'r') as f:
        data = json.load(f)
    
    dates = []
    scores = []
    for item in data:
        # Extract the pass dictionary
        pass_data = item.get("pass", {})
        
        # Extract date info
        date_info = pass_data.get("date", {})
        year = date_info.get("year", 2026)
        month = date_info.get("month", 1)
        day = date_info.get("day", 1)
        dates.append(datetime(year, month, day))
        
        # Check if this pass is "No Production" (Assuming 'N' appears in assignments)
        assignments = pass_data.get("assignments", [])
        is_no_production = any(task == "N" for emp, task in assignments)
        
        if is_no_production:
            # Append NaN to break the line in matplotlib
            scores.append(np.nan)
        else:
            scores.append(pass_data.get("total", 0))
        
    return dates, scores

def plot_quality_scores(dates, scores_manual, scores_initial, scores_rolling):
    # Dynamic sizing based on timeline length - increased base size for larger fonts
    fig, ax = plt.subplots(figsize=(max(16, len(dates) * 0.35), 8))

    # Color palette requested
    colors = {
        "manual": "#e17674",   # Red
        "initial": "#fdbb84",  # Orange
        "rolling": "#a1d99b"   # Green
    }

    # Use a categorical integer index to ensure perfectly equal spacing between points
    x_indices = np.arange(len(dates))

    # --- SLIGHTLY INCREASED LINE WIDTH ---
    line_w = 5
    marker_s = 7

    # Plot the lines (matplotlib automatically breaks lines at np.nan)
    ax.plot(x_indices, scores_manual, color=colors["manual"], linewidth=line_w, marker='o', markersize=marker_s, zorder=3, label="Manual")
    ax.plot(x_indices, scores_initial, color=colors["initial"], linewidth=line_w, marker='o', markersize=marker_s, zorder=3, label="Initial (Open-Loop)")
    ax.plot(x_indices, scores_rolling, color=colors["rolling"], linewidth=line_w, marker='o', markersize=marker_s, zorder=3, label="Rolling (Closed-Loop)")

    # --- ADDED BACK THE NUMBERS BY THE DOTS ---
    for scores_list, color in [(scores_manual, colors["manual"]), 
                               (scores_initial, colors["initial"]), 
                               (scores_rolling, colors["rolling"])]:
        for x, y in zip(x_indices, scores_list):
            # SKIP drawing the text if the value is NaN
            if np.isnan(y):
                continue
                
            # ax.annotate(f"{y:.0f}", 
            #             (x, y), 
            #             textcoords="offset points", 
            #             xytext=(0, 8), 
            #             ha='center', 
            #             fontsize=12,
            #             fontweight='bold',
            #             color=color,
            #             zorder=4)

    # Clean up the axes by removing the outer box
    for spine in ax.spines.values():
        spine.set_visible(False)

    # Add a thin, visible grid and put it behind the lines
    ax.grid(color='#cccccc', linestyle='-', linewidth=0.5, zorder=1)
    ax.set_axisbelow(True)
    ax.tick_params(which="both", bottom=False, left=False)

    # Y-LABEL
    ax.set_ylabel("Assignment Quality Score", labelpad=15, fontsize=24)
    
    # Format X-axis to show DD.MM. periodically
    tick_spacing = max(1, len(dates) // 15)
    ticks_to_show = x_indices[::tick_spacing]
    labels_to_show = [dates[i].strftime("%d.%m.") for i in ticks_to_show]
    
    ax.set_xticks(ticks_to_show)
    
    # INCREASED TICK SIZES
    ax.set_xticklabels(labels_to_show, fontsize=24, rotation=45)
    ax.tick_params(axis='y', labelsize=24)

    # --- REMOVED BOLD FROM MAIN TITLE ---
    fig.suptitle("Volvo GTO Assignment Quality Evaluation for November 2025", fontsize=38, y=0.99)
    
    # --- REDUCED GAP BETWEEN LEGEND AND PLOT ---
    # Legend is positioned at 0.955, and the plot canvas reaches up to 0.945 to make them very close
    handles, labels = ax.get_legend_handles_labels()
    fig.legend(handles, labels, loc='upper center', ncol=3, frameon=False, fontsize=24, bbox_to_anchor=(0.5, 0.955))

    plt.tight_layout(rect=[0, 0, 1, 0.945])
    plt.savefig("assignment_quality_evaluation_november_tau_8.pdf", bbox_inches="tight")
    plt.show()

if __name__ == "__main__":
    path_manual = "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/GTO/GTO_november_manual_tau_8.json"
    path_initial = "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/GTO/GTO_november_initial_tau_8.json"
    path_rolling = "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/GTO/GTO_november_rolling_tau_8.json"

    # path_manual = "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/GTO/GTO_january_manual_tau_8.json"
    # path_initial = "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/GTO/GTO_january_initial_tau_8.json"
    # path_rolling = "/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/GTO/GTO_january_rolling_tau_8.json"

    # 1. Load data
    dates_m, scores_manual = load_evaluation_data(path_manual)
    dates_i, scores_initial = load_evaluation_data(path_initial)
    dates_r, scores_rolling = load_evaluation_data(path_rolling)

    # 2. Safety check for lengths
    min_len = min(len(dates_m), len(dates_i), len(dates_r))
    if len(dates_m) != len(dates_i) or len(dates_m) != len(dates_r):
        print("Warning: Your JSON files have different lengths. Truncating to the shortest one.")
    
    # Slice lists to equal length
    dates_m = dates_m[:min_len]
    scores_manual = scores_manual[:min_len]
    scores_initial = scores_initial[:min_len]
    scores_rolling = scores_rolling[:min_len]

    # 3. SYNCHRONIZE GAPS: If manual is NaN, force initial and rolling to also be NaN
    for i in range(min_len):
        if np.isnan(scores_manual[i]):
            scores_initial[i] = np.nan
            scores_rolling[i] = np.nan
    
    # 4. Plot
    plot_quality_scores(dates_m, scores_manual, scores_initial, scores_rolling)