import json
import random
import string

def generate_random_name():
    """
    Generate a random 'name' for demonstration,
    e.g. 'Alice' or 'Bob'. In real usage, you might
    have a predefined list or more advanced logic.
    """
    # Just pick a random capital letter + some letters
    # e.g. "Op" + random letters => "OpAxy"
    # Or pick from a small sample of known names if you prefer.
    # For simplicity, let's pick from a small, re-usable list:
    candidate_names = [
        "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Heidi",
        "Ivan", "Judy", "Kelly", "Luis", "Maria", "Nathan", "Olivia", "Paul",
        "Quentin", "Rosa", "Steve", "Trudy", "Uma", "Victor", "Wanda", "Xavier",
        "Yvonne", "Zach", "Brad", "Charlie", "Diana", "Eric", "Felicia",
        "George", "Hannah", "Irene", "Jack", "Karl", "Linda", "Andrea", "Tina",
        "Ingrid", "John"
    ]
    return random.choice(candidate_names)

def generate_operations(num_operations, operation_score_min=1, operation_score_max=10):
    """
    Generate a dictionary of operations for a station.
    Keys: 'O1', 'O2', ..., 'O<num_operations>'
    Values: random integer scores between operation_score_min and operation_score_max
    """
    ops = {}
    for i in range(1, num_operations + 1):
        op_name = f"O{i}"
        score = random.randint(operation_score_min, operation_score_max)
        ops[op_name] = score
    return ops

def random_subset(lst):
    """
    Return a random subset of the given list (possibly empty, possibly full).
    """
    subset = []
    for x in lst:
        # 50% chance to include each item
        if random.random() < 0.5:
            subset.append(x)
    return subset

def generate_people(
    num_operators,
    operations,
    team_leader=True,
    team_leader_name=None,
    team_leader_competences=2
):
    """
    Generate a list of 'people' (operators + optional 1 team leader).
    
    :param num_operators: number of OPERATOR roles
    :param operations: list of operation names (e.g. ["O1","O2","O3"])
    :param team_leader: if True, add one TEAM_LEADER
    :param team_leader_name: optional, fix the team leader's name
    :param team_leader_competences: how many operations the team leader is usually competent in
    :return: list of dicts with 'name', 'role', 'competences', 'preferences'
    """
    people = []

    # Optionally add team leader
    if team_leader:
        name = team_leader_name if team_leader_name else generate_random_name()
        # Pick a random subset of operations of fixed size (team_leader_competences)
        # so that the team leader has fewer competences
        leader_ops = random.sample(operations, min(team_leader_competences, len(operations)))
        # Let's say the team leader prefers a random subset of these, or maybe just 1
        leader_prefs = random_subset(leader_ops)
        person = {
            "name": name,
            "role": "TEAM_LEADER",
            "competences": leader_ops,
            "preferences": leader_prefs
        }
        people.append(person)

    # Generate operators
    for _ in range(num_operators):
        op_name = generate_random_name()
        # Make a random subset of operations as competences
        # This can range from empty to full, but let's ensure at least 1 for realism
        comp_subset = random_subset(operations)
        if not comp_subset:
            comp_subset = [random.choice(operations)]
        
        # Preferences must be a subset of competences
        prefs_candidates = random_subset(comp_subset)
        
        person = {
            "name": op_name,
            "role": "OPERATOR",
            "competences": comp_subset,
            "preferences": prefs_candidates
        }
        people.append(person)

    return people

def generate_station(name, num_operations, num_operators, use_team_leader=True):
    """
    Generate a station dictionary with:
    - operations (random scores)
    - people (1 team leader optionally + num_operators)
    """
    # 1) create operations
    ops_dict = generate_operations(num_operations)

    # 2) create people
    #   - operations list is the keys from ops_dict
    ops_list = list(ops_dict.keys())
    people_list = generate_people(
        num_operators,
        ops_list,
        team_leader=use_team_leader,
        team_leader_competences=2  # or some configurable number
    )

    station = {
        "operations": ops_dict,
        "people": people_list
    }
    return station

def generate_data(
    num_stations=4,
    operations_per_station=(7, 9, 6, 8),
    operators_per_station=(8, 10, 7, 9),
    use_team_leader=True
):
    """
    Generate the entire data structure with multiple stations.

    :param num_stations: how many stations to generate
    :param operations_per_station: tuple or list with the number of operations for each station
        if len(operations_per_station) < num_stations, we cycle or clamp.
    :param operators_per_station: tuple or list with how many operators each station has
        similarly handled as above
    :param use_team_leader: bool, if True each station has a team leader
    """
    stations = {}
    
    for i in range(num_stations):
        st_name = f"S{i+1}"
        # pick number of ops from the array, or clamp
        ops_count = operations_per_station[i % len(operations_per_station)]
        # pick number of operators
        opers_count = operators_per_station[i % len(operators_per_station)]

        station_dict = generate_station(
            st_name,
            num_operations=ops_count,
            num_operators=opers_count,
            use_team_leader=use_team_leader
        )
        stations[st_name] = station_dict

    return {
        "stations": stations
    }

def main():
    # Example usage with default parameters
    random.seed(42)  # for reproducibility, if desired
    
    # Here we define how many stations and how many ops/operators each
    # If you want each station to have the same number, you can do:
    # operations_per_station = [7]*4  (4 stations, each 7 ops)
    # operators_per_station = [8]*4   (4 stations, each 8 operators)
    
    data = generate_data(
        num_stations=4,
        operations_per_station=(7, 9, 6, 8),
        operators_per_station=(8, 10, 7, 9),
        use_team_leader=True
    )

    # Write to JSON file
    with open("competences_preferences_generated.json", "w") as f:
        json.dump(data, f, indent=2)

    print("JSON file 'competences_preferences_generated.json' created!")

if __name__ == "__main__":
    main()
