import json
import matplotlib.pyplot as plt

def plot_points(data):
    fig, ax = plt.subplots()

    # We can pick 10 distinct colors from a colormap
    color_map = plt.get_cmap("tab10")

    # For each zone/key, we have a list of points: (omega, alpha).
    for i, entry in enumerate(data):
        key_id = entry["key_id"]
        points = entry["points"]

        # Extract separate lists for omega and alpha
        omegas = [p[0] for p in points]
        alphas = [p[1] for p in points]

        # Pick a color from the colormap
        color = color_map(i % 10)

        # Plot them with a scatter
        ax.scatter(omegas, alphas, s=200, color=color, label=key_id, alpha=0.8)


        # for (x, y) in zip(omegas, alphas):
        #     ax.text(
        #         x, y,
        #         key_id,        # The label (e.g., your key_id)
        #         color="black",
        #         fontsize=10,
        #         ha="center",
        #         va="center"
        #     )

    ax.set_xlabel("omega")
    ax.set_ylabel("alpha")
    ax.set_title("Zones from Rust JSON (Points Only)")
    ax.set_aspect("equal", adjustable="box")

    # Fix axes if you want, or let matplotlib auto-scale
    ax.set_xlim(-1, 10)
    ax.set_ylim(-1, 10)

    # Show legend with the key IDs
    ax.legend(
        bbox_to_anchor=(1.03, 1),  # Move the legend slightly off the right edge
        loc='upper left',          # Position the legend's upper left corner at that anchor
        markerscale=0.6
    )

    # ax.legend()
    plt.show()

def main():
    with open("/home/endre/Desktop/points.json", "r", encoding="utf-8") as f:
        data = json.load(f)

    plot_points(data)

if __name__ == "__main__":
    main()
