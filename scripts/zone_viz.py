import json
import matplotlib.pyplot as plt
import matplotlib.patches as patches

def plot_zones(data):
    fig, ax = plt.subplots()

    # We can pick 10 distinct colors from a colormap
    color_map = plt.get_cmap("tab10")

    for i, entry in enumerate(data):
        key_id = entry["key_id"]
        rects = entry["rects"]

        # Choose a color for this key
        color = color_map(i % 10)

        for rect in rects:
            om_min = rect["omega_min"]
            om_max = rect["omega_max"]
            al_min = rect["alpha_min"]
            al_max = rect["alpha_max"]
            
            width = om_max - om_min
            height = al_max - al_min
            
            # Fill the rectangle with the chosen color
            patch = patches.Rectangle(
                (om_min, al_min),
                width,
                height,
                facecolor=color,  # Fill color
                alpha=0.3,        # Slight transparency
                edgecolor=color,  # Outline color matches fill
                linewidth=1
            )
            ax.add_patch(patch)

            # Optionally label the rectangle with the key_id in the center
            ax.text(
                (om_min + om_max) / 2,
                (al_min + al_max) / 2,
                key_id,
                color="black",
                ha='center',
                va='center',
                fontsize=8,
                alpha=0.8
            )

    # Fix the axes to go from 0 to 10
    ax.set_xlim(0, 26)
    ax.set_ylim(0, 26)
    ax.set_xlabel("omega")
    ax.set_ylabel("alpha")
    ax.set_title("Solution Zones")
    ax.set_aspect("equal", adjustable="box")  # make squares look square

    plt.show()

def main():
    with open("/home/endre/Desktop/zones.json", "r", encoding="utf-8") as f:
        data = json.load(f)

    plot_zones(data)

if __name__ == "__main__":
    main()
