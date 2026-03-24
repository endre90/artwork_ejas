import json
import numpy as np
import matplotlib.pyplot as plt
from datetime import date
from collections import deque

def load_data(history_path, matrix_path, station_name="CE"):
    with open(history_path, "r") as f:
        history_data = json.load(f)
        
    with open(matrix_path, "r") as f:
        matrix_data = json.load(f)
        station = matrix_data.get("stations", {}).get(station_name, {})
        
    return history_data, station

def calculate_daily_quality_scores(history_data, station, params):
    # Unpack parameters
    omega = params.get("omega", 1)
    alpha = params.get("alpha", 192)
    beta = params.get("beta", 384)
    gamma = params.get("gamma", 24)
    tau = params.get("tau", 5)
    
    # Extract station rules
    ergo_scores = station.get("ergo_score", {})
    max_ergo = max(ergo_scores.values()) if ergo_scores else 5
    jobs = sorted(list(ergo_scores.keys()))
    
    # Map operator names to their preference lists
    people_prefs = {p["name"]: p.get("preferences", []) for p in station.get("people", [])}
    
    # Sort history chronologically
    data_sorted = sorted(history_data, key=lambda x: (
        x["day"]["date"]["year"], x["day"]["date"]["month"], x["day"]["date"]["day"]
    ))
    
    dates = []
    scores = []
    
    # Rolling window to track the previous tau active days
    history_window = deque(maxlen=tau)
    
    for day_data in data_sorted:
        day_info = day_data["day"]
        dt = date(day_info["date"]["year"], day_info["date"]["month"], day_info["date"]["day"])
        dates.append(dt)
        
        # 1. Build h_matrix (historical count) from the rolling window
        h_matrix = {p: {j: 0 for j in jobs} for p in people_prefs.keys()}
        for past_day in history_window:
            for op, job in past_day["assignments"]:
                if op in h_matrix and job in jobs:
                    h_matrix[op][job] += 1
                    
        # 2. Calculate daily penalty/reward terms
        pref_reward = 0
        leader_penalty = 0
        external_penalty = 0
        historical_penalty = 0
        
        leader = day_info.get("leader")
        
        day = 0
        for op, job in day_info["assignments"]:
            day = day + 1
            # Check for non-standard task codes
            # if job == "E":  
                # external_penalty += 1
                # continue
            if job in ["E", "T", "L"]:  # Absent, Training or Loaned out
                continue
                
            # Score standard assignments
            if op in people_prefs and job in jobs:
                # A. Preference Score
                prefs = people_prefs[op]
                rank = prefs.index(job) if job in prefs else len(jobs)
                pref_reward += (len(jobs) - rank)
                
                # B. Leader Penalty
                # No leader penalty for VCE 
                # if op == leader:
                #     leader_penalty += 1
                    
                # C. Historical & Ergonomic Penalty
                hist_count = h_matrix[op][job]
                e_j = ergo_scores.get(job, 1)
                ergo_multiplier = max_ergo - e_j + 1
                historical_penalty += (hist_count * ergo_multiplier)
            # else:
                # If job isn't a standard station job, count as external/unassigned
                # external_penalty += 1
        print("day", day)
        print("pref: ", pref_reward, "*", omega, "=", omega * pref_reward)
        print("lead: ", leader_penalty, "*", alpha, "=", alpha * leader_penalty)
        print("exte: ", external_penalty, "*", beta, "=", beta * external_penalty)
        print("hist/ergo: ", historical_penalty, "*", gamma, "=", gamma * historical_penalty)
                
        # 3. Calculate Final DQS for the day
        daily_score = 1000 + (omega * pref_reward) - (alpha * leader_penalty) - (beta * external_penalty) - (gamma * historical_penalty)
        scores.append(daily_score)
        
        # 4. Add the current day to the rolling history window for the next day's calculation
        history_window.append(day_info)
        
    return dates, scores

def plot_quality_scores(dates, scores):
    # Dynamic sizing based on timeline length
    fig, ax = plt.subplots(figsize=(max(10, len(dates) * 0.25), 5))

    # Color the line using your palette
    color_line = "#e17674"

    # Use a categorical integer index to ensure perfectly equal spacing between points
    x_indices = np.arange(len(dates))

    # Plot the line (no fill)
    ax.plot(x_indices, scores, color=color_line, linewidth=2.5, marker='o', markersize=6, zorder=3)

    # Add small numbers by each dot
    for x, y in zip(x_indices, scores):
        # Using offset points ensures the text hovers exactly 8 pixels above the dot
        ax.annotate(f"{y:.0f}", 
                    (x, y), 
                    textcoords="offset points", 
                    xytext=(0, 8), 
                    ha='center', 
                    fontsize=9,
                    color="#444444",
                    zorder=4)

    # Clean up the axes by removing the outer box
    for spine in ax.spines.values():
        spine.set_visible(False)

    # Add a thin, visible grid and put it behind the line (zorder=1)
    ax.grid(color='#cccccc', linestyle='-', linewidth=0.5, zorder=1)
    ax.set_axisbelow(True)
    ax.tick_params(which="both", bottom=False, left=False)

    # Labels and Titles
    ax.set_ylabel("Daily Quality Score (DQS)", labelpad=15, fontsize=13)
    
    # Format X-axis to show DD.MM. periodically to avoid overcrowding
    tick_spacing = max(1, len(dates) // 15)
    ticks_to_show = x_indices[::tick_spacing]
    labels_to_show = [dates[i].strftime("%d.%m.") for i in ticks_to_show]
    
    ax.set_xticks(ticks_to_show)
    ax.set_xticklabels(labels_to_show, fontsize=11)
    
    # Tilt the y-axis ticks by 45 degrees
    plt.yticks(fontsize=11)
    plt.xticks(fontsize=11, rotation=45)

    ax.set_title("Evaluation of Manual Assignment Quality Over Time", fontsize=16, pad=25)

    # Increase top margin slightly so annotations on peak dots don't get cut off
    plt.tight_layout(rect=[0, 0.05, 1, 0.95])
    plt.savefig("manual_quality_evaluation.pdf", bbox_inches="tight")
    plt.show()

def main():
    # Update these paths to point to your local files
    # history_file = "/home/endre/rust_ws/artwork_ejas/data/factory/VCE_history_part_a_temp.json"
    history_file = "/home/endre/rust_ws/artwork_ejas/data/factory/VCE_algo_strat_1_daily_part_a.json"
    matrix_file = "/home/endre/rust_ws/artwork_ejas/data/factory/VCE_matrix.json" 
    
    # Load data
    history_data, station = load_data(history_file, matrix_file, station_name="CE")
    
    # The weights you defined in your Rust model
    params = {
        "omega": 1,
        "alpha": 192,
        "beta": 384,
        "gamma": 24,
        "tau": 5
    }
    
    dates, scores = calculate_daily_quality_scores(history_data, station, params)
    
    # Print the average score for your paper
    print(f"Average Manual Daily Quality Score: {np.mean(scores):.2f}")
    
    # Generate the plot
    plot_quality_scores(dates, scores)

if __name__ == "__main__":
    main()