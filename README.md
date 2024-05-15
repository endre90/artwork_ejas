# Artwork: Employee-job assignment problem

The optimization model to assign \( N \) workers to \( M \) jobs, taking into account competences and preferences, is defined as follows:

### Objective Function

Maximize the total preference score:

$$
\text{Maximize} \quad \sum_{i=1}^{N} \sum_{j=1}^{M} (M - P_{ij}) \cdot x_{ij}
$$

### Constraints

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

4. Binary decision variables:

$$
x_{ij} \in \{0, 1\} \quad \forall i, j
$$


### Alternative constraints

1. Each worker is assigned to at most one job:

$$
\sum_{j=1}^{M} x_{ij} \leq 1 \quad \forall i \in \{1, \ldots, N\}
$$

2. Combined constraint with implication: If worker \( i \) is assigned to job \( j \), then:
   - No other worker \( k \) can be assigned to job \( j \) (for \( k \neq i \)).
   - Worker \( i \) cannot be assigned to another job \( l \) (for \( l \neq j \)).
   - Worker \( i \) must be competent to perform job \( j \).

$$
x_{ij} \implies \left( \bigwedge_{k \neq i} \neg x_{kj} \right) \land \left( \bigwedge_{l \neq j} \neg x_{il} \right) \land (x_{ij} \leq C_{ij}) \quad \forall i, j
$$

3. Binary decision variables:

$$
x_{ij} \in \{0, 1\} \quad \forall i, j
$$