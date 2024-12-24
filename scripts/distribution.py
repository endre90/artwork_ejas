import json
import matplotlib.pyplot as plt
import numpy as np

# Predefined operations per station
STATION_OPS = {
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
            label = f'{height:.{decimals}f}' if decimals > 0 else f'{int(height)}'
            ax.annotate(label,
                        xy=(rect.get_x() + rect.get_width()/2, height),
                        xytext=(0, 3),  # 3 points vertical offset
                        textcoords="offset points",
                        ha='center', va='bottom', fontsize=8)

def plot_station(station_name, station_data):
    operations = STATION_OPS[station_name]
    people = station_data["people"]
    ergonomics = station_data["operations"]  # Ergonomics scores from file

    competence_count, preference_count = count_competences_and_preferences(people, operations)

    x = np.arange(len(operations))
    width = 0.2  # We have 3 bars

    fig, ax = plt.subplots(figsize=(6, 4))
    # fig.suptitle(f"Station {station_name}: Competences, Preferences & Ergonomics", fontsize=14)

    # Plot competences (green)
    rects1 = ax.bar(x - width, [competence_count[op] for op in operations], width, label='Competences', color='green')
    # Plot preferences (purple)
    rects2 = ax.bar(x, [preference_count[op] for op in operations], width, label='Preferences', color='purple')
    # Plot ergonomics (orange)
    rects3 = ax.bar(x + width, [ergonomics[op] for op in operations], width, label='Ergonomics', color='orange')

    ax.set_xticks(x)
    ax.set_xticklabels(operations)
    ax.set_ylabel("Count / Ergonomics Score")
    ax.set_xlabel("Operations")

    # Add value labels
    add_value_labels(ax, rects1, decimals=0)  # competences: int
    add_value_labels(ax, rects2, decimals=0)  # preferences: int
    add_value_labels(ax, rects3, decimals=0)  # ergonomics: one decimal

    # Set y-limit to max(total_people, 10) to ensure ergonomics is visible
    total_people = len(people)
    y_max = max(total_people, 10)
    # Also consider if some ergonomics score might exceed 10 (if assigned that way),
    # we can ensure the limit is at least max(ergonomics)+1
    max_erg = max(ergonomics.values())
    y_max = max(y_max, max_erg + 1)

    ax.set_ylim(0, y_max)

    ax.legend(loc='upper right')

    plt.tight_layout(rect=[0, 0.03, 1, 0.95])
    plt.show()

def main():
    # Load data
    with open("/home/endre/rust_crates/artwork_ejas/data/matrix.json", "r") as f:
        data = json.load(f)

    stations = data["stations"]

    # Plot each station individually
    for station_name in ["S1", "S2", "S3", "S4"]:
        plot_station(station_name, stations[station_name])

if __name__ == "__main__":
    main()







