import json
import matplotlib.pyplot as plt
import numpy as np

# Predefined operations per station
STATION_OPS = {
    "GTO": ["O1","O2","O3","O4","O5","O6","O7","O8"],
    "CE": ["O1", "O2", "O3", "O4", "O5", "O6", "O7", "O8", "O9", "O10", "O11", "O12"],
    "S1": ["O1","O2","O3","O4","O5","O6","O7"],
    "S2": ["O1","O2","O3","O4","O5","O6","O7","O8","O9"],
    "S3": ["O1","O2","O3","O4","O5","O6"],
    "S4": ["O1","O2","O3","O4","O5","O6","O7","O8"]
}

def count_competences_and_preferences(people, operations):
    # Initialize counts
    competence_count = {op: 0 for op in operations}
    preference_count = {op: 0 for op in operations}

    for person in people:
        for op in person["competences"]:
            if op in competence_count:
                competence_count[op] += 1
        for op in person["preferences"]:
            if op in preference_count:
                preference_count[op] += 1
    
    return competence_count, preference_count

def add_value_labels(ax, rects, decimals=0):
    # Attach a text label above each bar displaying its height.
    for rect in rects:
        height = rect.get_height()
        if height > 0:
            # Forcing rounding then integer conversion to cleanly drop floating points
            label = f'{height:.{decimals}f}' if decimals > 0 else f'{int(round(height))}'
            ax.annotate(label,
                        xy=(rect.get_x() + rect.get_width()/2, height),
                        xytext=(0, 4),  # 4 points vertical offset
                        textcoords="offset points",
                        ha='center', va='bottom', fontsize=9, fontweight='bold', color='#4a4a4a')

def plot_station(station_name, station_data):
    operations = STATION_OPS[station_name]
    people = station_data["people"]
    ergonomics = station_data["ergo_score"]  # Ergonomics scores from file

    competence_count, preference_count = count_competences_and_preferences(people, operations)

    x = np.arange(len(operations))
    width = 0.25  # Slightly wider bars for better visibility 

    fig, ax = plt.subplots(figsize=(10, 6))

    # Unified Color Palette - Light pastel tones
    c_comp = "#a1d99b"  # Soft Green
    c_pref = "#9e9ac8"  # Lighter Purple
    c_ergo = "#fdbb84"  # Lighter Orange

    # Plot bars with a clean white edge
    rects1 = ax.bar(x - width, [competence_count[op] for op in operations], width, label='Competences count', color=c_comp, edgecolor='white', linewidth=0.7)
    rects2 = ax.bar(x, [preference_count[op] for op in operations], width, label='Preferences count', color=c_pref, edgecolor='white', linewidth=0.7)
    rects3 = ax.bar(x + width, [ergonomics[op] for op in operations], width, label='Ergonomic scores', color=c_ergo, edgecolor='white', linewidth=0.7)

    # Styling Axes and Labels
    ax.set_xticks(x)
    ax.set_xticklabels(operations, fontsize=9)
    # ax.set_yticklabels(operations, fontsize=9)
    ax.set_ylabel("Count / Ergonomics Score", labelpad=10, fontsize=11)
    ax.set_xlabel("Operations", labelpad=10, fontsize=11)

    # Remove outer black box (spines) for a modern look
    for spine in ['top', 'right', 'left']:
        ax.spines[spine].set_visible(False)
    
    # Add subtle horizontal grid lines behind the bars
    ax.set_axisbelow(True)
    ax.grid(axis='y', color='#e0e0e0', linestyle='--', linewidth=1)
    ax.tick_params(axis='y', left=False) # Hide little tick lines on Y axis
    ax.tick_params(axis='x', bottom=False) # Hide little tick lines on X axis

    # Add value labels (decimals=0 for all to ensure integers only)
    add_value_labels(ax, rects1, decimals=0)  
    add_value_labels(ax, rects2, decimals=0)  
    add_value_labels(ax, rects3, decimals=0)  

    # Set y-limit padding
    total_people = len(people)
    y_max = max(total_people, 10)
    max_erg = max(ergonomics.values())
    y_max = max(y_max, max_erg + 1)
    ax.set_ylim(0, y_max * 1.15)

    # Move legend to a single horizontal line above the plot, centered
    ax.legend(loc='lower center', bbox_to_anchor=(0.5, 1.02), ncol=3, frameon=False, fontsize=10)

    # Increased padding on the title to leave room for the legend directly underneath it
    plt.title(f"Volvo {station_name} station: Competence & Preference count, and Ergonomic scores", fontsize=16, pad=35)
    
    plt.tight_layout()
    plt.show()

def main():
    # Load data
    with open("/home/endre/rust_ws/artwork_ejas/data/factory/GTO_matrix.json", "r") as f:
        data = json.load(f)

    stations = data["stations"]

    # Plot each station individually
    for station_name in ["GTO"]:
        plot_station(station_name, stations[station_name])

if __name__ == "__main__":
    main()