# Artwork use-case: Employee-job assignment system

The problem is to assign $N$ workers to $M$ jobs, taking into account competences and preferences.

## Quickstart
1. Install Rust: https://www.rust-lang.org/tools/install
2. Install Z3: https://github.com/Z3Prover/z3
3. Clone this repository
```
git clone https://github.com/endre90/artwork_ejas.git
```
4. Build code:
```
cd artwork_ejas
cargo build
```
5. Run an example, for instance:
```
TODO
```
## Background

The remaining text shows the development steps of the EJAS for Artwork.
1. Step 1: 
   - Equal number of jobs and employees
   - No planning horizon
   - No history taken into account
   - No fairness
   - No external employees
   - 

## Step 1: Static assignment
The static assignment assigns $N$ workers to $M$ jobs based on their competences and preferences to maximize overall satisfaction. Each worker has a list of competences (indicating which jobs they can perform) and a ranked list of job preferences. The procedure creates binary decision variables to indicate assignments, and it aims to maximize the total preference score while ensuring each worker is assigned to at most one job, each job is covered by at most one worker, and every job is covered by at least one worker that is competent to perform it. The goal is to find an optimal static assignment that respects workers' competences and maximizes their preferences.

### Step 1: Notation

- $N$: 
Number of workers.
- $M$: Number of jobs.
- $C_{ij}$: Binary competence matrix where $C_{ij} = 1$ if worker $i$ can perform job $j$, and $C_{ij} = 0$ otherwise.
- $P_{ij}$: Preference rank matrix where $P_{ij}$ is the preference rank of job $j$ for worker $i$. Lower values in $P_{ij}$ indicate higher preference.

### Step 1: Decision Variables

- $x_{ij}$: Binary decision variable such that:

$$
x_{ij} =
\begin{cases} 
1 & \text{if worker } i \text{ is assigned to job } j \\ 
0 & \text{otherwise} 
\end{cases}
$$

### Step 1: Objective Function

Maximize the total preference score:

$$
\text{Maximize} \quad \sum_{i=1}^{N} \sum_{j=1}^{M} (M - P_{ij}) \cdot x_{ij}
$$

### Step 1: Constraints

1. Each worker is assigned to at most one job:

$$
\sum_{j=1}^{M} x_{ij} \leq 1 \quad \forall i \in \{1, \ldots, N\}
$$

2. Each job is assigned to at most one worker:

$$
\sum_{i=1}^{N} x_{ij} \leq 1 \quad \forall j \in \{1, \ldots, M\}
$$

3. Only assign jobs that workers are competent to perform:

$$
x_{ij} \leq C_{ij} \quad \forall i, j
$$

4. Each job must be covered by at least one employee who is competent to perform it:

$$
\sum_{i=1}^{N} C_{ij} \geq 1 \quad \forall j \in \{1, \ldots, M\}
$$
