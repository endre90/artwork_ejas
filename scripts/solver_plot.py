import json
import matplotlib.pyplot as plt

def plot_solver_times(filepath):
    # Load the JSON data
    with open(filepath, 'r') as f:
        data = json.load(f)

    solver_times = []

    # Extract solver times only
    for item in data:
        pass_data = item.get("pass", {})
        
        # Parse solver time (remove 'ms' and convert to float)
        time_str = pass_data.get("solver_time", "0ms")
        time_val = float(time_str.replace("ms", ""))
        solver_times.append(time_val)

    # Increased height slightly to 3.5 to hold the massive text without clipping
    plt.figure(figsize=(15, 3.5))
    
    plt.plot(solver_times, marker='o', color="black", linewidth=1.0, markersize=4, zorder=3)
    
    # --- CHANGED: Even larger main title ---
    plt.title("Solver Time", fontsize=26, pad=15, y=1.0, color="#333333")
    
    # --- CHANGED: Even larger Y-axis label ---
    plt.ylabel("Time (ms)", fontsize=20, labelpad=20)
    
    # Hide X-axis ticks and labels completely
    plt.tick_params(axis='x', which='both', bottom=False, top=False, labelbottom=False)
    
    # --- CHANGED: Even larger Y-axis numbers (ticks) ---
    plt.tick_params(axis='y', labelsize=18)
    
    # Clean up the grid and borders
    ax = plt.gca()
    
    # Only show horizontal grid lines
    ax.grid(axis='y', color='#cccccc', linestyle='--', linewidth=0.5, zorder=1)
    ax.set_axisbelow(True)
    
    # Hide top, right, and bottom borders
    ax.spines['top'].set_visible(False)
    ax.spines['right'].set_visible(False)
    ax.spines['bottom'].set_visible(False) 

    # Adjust layout to fit the new large text
    plt.tight_layout()
    
    # Show the plot
    plt.show()

if __name__ == "__main__":
    # Point this to your JSON file
    plot_solver_times("/home/endre/rust_ws/artwork_ejas/data/factory/evaluation/solver_times/all_times.json")