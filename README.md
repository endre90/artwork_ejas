# Artwork use-case: Employee-job assignment system

The problem is to assign $N$ employees to $M$ jobs, taking into account competences and preferences.

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
2. Step 2:
   - The number of jobs and employees doesn't have to be equal. Sometimes, there will be a surplus of employees that will remain unnasigned, and sometimes there will be a deficit of employees, in which case we have to pull in external employees to do the job, however, this costs a lot an should be avoided.

## Step 1: Static assignment
The static assignment assigns $N$ employees to $M$ jobs based on their competences and preferences to maximize overall satisfaction. Each employee has a list of competences (indicating which jobs they can perform) and a ranked list of job preferences. The procedure creates binary decision variables to indicate assignments, and it aims to maximize the total preference score while ensuring each employee is assigned to at most one job, each job is covered by at most one employee, and every job is covered by at least one employee that is competent to perform it. The goal is to find an optimal static assignment that respects employees' competences and maximizes their preferences.

### Notation

- $N$: 
Number of employees
- $M$: Number of jobs
- $C_{ij}$: Binary competence matrix where $C_{ij} = 1$ if employee $i$ can perform job $j$, and $C_{ij} = 0$ otherwise
- $P_{ij}$: Preference rank matrix where $P_{ij}$ is the preference rank of job $j$ for employee $i$. Lower values (taken as index) in $P_{ij}$ indicate higher preference

### Decision Variables

- $x_{ij}$: Binary decision variable such that:

$$
x_{ij} =
\begin{cases} 
1 & \text{if employee } i \text{ is assigned to job } j \\ 
0 & \text{otherwise} 
\end{cases}
$$

### Objective Function

Maximize the total preference score:

$$
\text{Maximize} \quad \sum_{i=1}^{N} \sum_{j=1}^{M} (M - P_{ij}) \cdot x_{ij}
$$

### Constraints

1. Each employee is assigned to at most one job:

$$
\sum_{j=1}^{M} x_{ij} \leq 1 \quad \forall i \in \{1, \ldots, N\}
$$

2. Each job is assigned to exactly one employee:

$$
\sum_{i=1}^{N} x_{ij} = 1 \quad \forall j \in \{1, \ldots, M\}
$$

3. Only assign jobs to competent employees:

$$
x_{ij} \implies C_{ij} \quad \forall i, j
$$

4. Each job must be covered by at least one employee who is competent to perform it:

$$
\sum_{i=1}^{N} C_{ij} \cdot x_{ij} \geq 1 \quad \forall j \in \{1, \ldots, M\}
$$

## Step 2: External assignment
In this step, the static assignment problem is extended with the possibility to have more jobs than employees, or to have jobs for which not enough competences exist. In this case, external employees can be allocated to complete the job, however this has a high cost and should be avoided if possible.

### Notation

- $N$: 
Number of employees.
- $M$: Number of jobs.
- $C_{ij}$: Binary competence matrix where $C_{ij} = 1$ if employee $i$ can perform job $j$, and $C_{ij} = 0$ otherwise.
- $P_{ij}$: Preference rank matrix where $P_{ij}$ is the preference rank of job $j$ for employee $i$. Lower values in $P_{ij}$ indicate higher preference.

### Decision Variables

- $x_{ij}$: Binary decision variable such that:

$$
x_{ij} =
\begin{cases} 
1 & \text{if employee } i \text{ is assigned to job } j \\ 
0 & \text{otherwise} 
\end{cases}
$$

- $e_j$: Binary decision variable such that:

$$
e_j =
\begin{cases} 
1 & \text{if an external employee is assigned to job } j \\ 
0 & \text{otherwise} 
\end{cases}
$$

### Objective Function

Maximize the total preference score while minimizing the use of external employees:

$$
\text{Maximize} \quad \sum_{i=1}^{N} \sum_{j=1}^{M} (M - P_{ij}) \cdot x_{ij} - \lambda \sum_{j=1}^{M} e_j
$$

where $\lambda$ is a penalty factor for using external employees.

### Constraints

1. Each internal employee is assigned to at most one job:

$$
\sum_{j=1}^{M} x_{ij} \leq 1 \quad \forall i \in \{1, \ldots, N\}
$$

2. Each job is assigned to either one internal employee or one external employee:

$$
\sum_{i=1}^{N} x_{ij} + e_j = 1 \quad \forall j \in \{1, \ldots, M\}
$$

3. Only assign jobs to competent employees:

$$
x_{ij} \implies C_{ij} \quad \forall i, j
$$

4. Each job must be covered by at least one competent employee or one external employee:

$$
\sum_{i=1}^{N} C_{ij} \cdot x_{ij} + e_j \geq 1 \quad \forall j \in \{1, \ldots, M\}
$$

## Step 3: Fair assignment with horizon

This model ensures that employees are fairly rotated through their highly preferred jobs over a 2-week period, avoiding repeated assignments to their least preferred jobs. The fairness constraint is dynamically adjusted by the heuristic parameter \( k \), which defines the percentage of top preferred jobs considered for fair distribution. This promotes a balanced distribution of job assignments based on the specified preference percentage. The objective function balances maximizing the total preference score, minimizing the use of external employees, and promoting fairness in job assignments.

- $N$: Number of internal employees.
- $M$: Number of jobs.
- $T$: Number of time slots (e.g., 14 days).
- $C_{ij}$: Binary competence matrix where $C_{ij} = 1$ if worker $i$ can perform job $j$, and $C_{ij} = 0$ otherwise.
- $P_{ij}$: Preference rank matrix where $P_{ij}$ is the preference rank of job $j$ for worker $i$. Lower values in $P_{ij}$ indicate higher preference.
- $x_{ijt}$: Binary decision variable such that:
  $$
  x_{ijt} =
  \begin{cases} 
  1 & \text{if worker } i \text{ is assigned to job } j \text{ at time } t \\ 
  0 & \text{otherwise} 
  \end{cases}
  $$
- $e_{jt}$: Binary decision variable such that:
  $$
  e_{jt} =
  \begin{cases} 
  1 & \text{if an external employee is assigned to job } j \text{ at time } t \\ 
  0 & \text{otherwise} 
  \end{cases}
  $$
- $k$: Heuristic parameter representing the percentage of top preferred jobs to consider (1 to 100).
- $\lambda$: Penalty factor for using external employees.
- $\beta$: Weight for the fairness term.
- $\delta$: Small tolerance value for the fairness constraint.

### Objective Function

The objective function can be expressed as:

$$
\text{Maximize} \quad \Phi - \Psi - \Gamma - \Delta
$$

<!-- $$
\text{Maximize} \quad \underbrace{\sum_{i=1}^{N} \sum_{j=1}^{M} \sum_{t=1}^{T} (M - P_{ij}) \cdot x_{ijt}}_{\text{Total Preference Score}} - \underbrace{\lambda \sum_{j=1}^{M} \sum_{t=1}^{T} e_{jt}}_{\text{Penalty for External Employees}} - \underbrace{\beta \sum_{i=1}^{N} \left| \sum_{j=1}^{M} \sum_{t=1}^{T} \left( P_{ij} \leq \left\lfloor \frac{k \cdot M}{100} \right\rfloor \right) \cdot x_{ijt} - \frac{T \cdot \left\lfloor \frac{k \cdot M}{100} \right\rfloor}{N} \right|}_{\text{Fairness Term 1: Deviation from Ideal Distribution}} - \underbrace{\beta \sum_{i=1}^{N} \sum_{j=1}^{M} \sum_{t=1}^{T} P_{ij} \cdot x_{ijt}}_{\text{Fairness Term 2: Minimization of Least Preferred Jobs}}
$$ -->

Where the parts are:

1. $\Phi$ - Total Preference Score :
   - Maximize the assignments to jobs that workers prefer.
   - $(M - P_{ij})$ ensures higher scores for more preferred jobs since $P_{ij}$ is lower for higher preferred jobs (taken as index).

$$
\Phi = \sum_{i=1}^{N} \sum_{j=1}^{M} \sum_{t=1}^{T} (M - P_{ij}) \cdot x_{ijt}
$$ 
   

2. $\Psi$ - Penalty for External Employees:
   - Penalize the use of external employees to minimize their usage.
   - $\lambda$ is a penalty factor that can be adjusted to change the emphasis on minimizing external worker usage.
   - TODO: Determine a good value for $\lambda$.

$$
\Psi = \lambda \sum_{j=1}^{M} \sum_{t=1}^{T} e_{jt}
$$

3. $\Gamma$ - Fairness Term 1: Deviation from Ideal Distribution:
   - Promote fair distribution of top $k$% preferred jobs.
   - Ensure each worker gets an approximately equal share of their top $k$% preferred jobs.
   - $\beta$ is a weight that can be adjusted to change the emphasis on fairness.
   - The term $\sigma$ calculates the number of jobs that are within the top $k$% preferences for each worker by multiplying the total number of jobs $M$ by $k/100$​ to get the fraction of jobs that are in the top $k$% and applying the floor function to round down to the nearest integer, ensuring we get a whole number of jobs.
   - TODO: Determine a good value for $\beta$.
   - TODO: Determine a good value for $k$.

$$
\Gamma = \beta \sum_{i=1}^{N} \left \| \sum_{j=1}^{M} \sum_{t=1}^{T} \left( P_{ij} \leq \sigma \right) \cdot x_{ijt} - \frac{T \cdot \sigma}{N} \right\|, \quad \sigma=\left\lfloor \frac{k \cdot M}{100} \right\rfloor
$$

4. $\Delta$ - Fairness Term 2: Minimization of Least Preferred Jobs:
   - Minimize the assignments of least preferred jobs.
   - $\beta$ is a weight that can be adjusted to change the emphasis fairness.

$$
\Delta = \beta \sum_{i=1}^{N} \sum_{j=1}^{M} \sum_{t=1}^{T} P_{ij} \cdot x_{ijt}
$$

### Constraints

1. Each internal worker is assigned to at most one job per time slot:

$$
\sum_{j=1}^{M} x_{ijt} \leq 1 \quad \forall i \in \{1, \ldots, N\}, \forall t \in \{1, \ldots, T\}
$$

2. Each job is assigned to either one internal worker or one external worker per time slot:

$$
\sum_{i=1}^{N} x_{ijt} + e_{jt} = 1 \quad \forall j \in \{1, \ldots, M\}, \forall t \in \{1, \ldots, T\}
$$

3. Only assign jobs to competent employees:

$$
x_{ijt} \implies C_{ijt} \quad \forall i, j, t
$$

4. Each job must be covered by at least one competent internal worker or an external worker:

$$
\sum_{i=1}^{N} C_{ij} \cdot x_{ijt} + e_{jt} \geq 1 \quad \forall j \in \{1, \ldots, M\}, \forall t \in \{1, \ldots, T\}
$$