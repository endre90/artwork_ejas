# Artwork use-case: Employee-job assignment system

The problem is to assign $N$ employees to $M$ jobs, taking into account competences and preferences.

## Quickstart

The app runs in your browser: an egui/eframe interface compiled to WebAssembly,
served by a small Rust server that runs the Z3 solver. Z3 is C++ and cannot be
compiled to WebAssembly, which is why the solving happens server-side.

### Run with Docker on Linux

No Rust, no Z3, no toolchain — just Docker.

```
git clone https://github.com/endre90/artwork_ejas.git
cd artwork_ejas
docker compose up
```

Then open <http://localhost:8080>.

- Stop it with `Ctrl-C`, or `docker compose down`.
- After pulling new changes: `docker compose up --build`.
- The port is set in `docker-compose.yml`. It is published as
  `127.0.0.1:8080:8080`, so the app is reachable only from this machine —
  there is no authentication, so do not expose it to a network without
  putting something in front of it.

**On the build time.** The first build takes a few minutes and is dominated by
compiling Rust dependencies. Z3 itself is *not* compiled: the image installs
the distro's prebuilt `libz3-dev` package (about 8 MB, a few seconds). The `z3`
crate is declared without the `bundled` / `static-link-z3` feature, so it links
dynamically against that system library. If you have ever waited an hour for Z3
to build from source, that is the feature you want to keep switched off.
Rebuilds after the first are cached and take well under a minute.

### Run with Docker on Windows

**Docker Desktop is the only thing to install**, and then the commands above
work unchanged. The image is Linux-based, but Docker Desktop runs it in a Linux
VM it manages entirely by itself.

1. Install **Docker Desktop for Windows**:
   <https://docs.docker.com/desktop/setup/install/windows-install/>
   Keep the default WSL 2 backend; the installer enables the Windows features
   it needs. Installing requires administrator rights.
2. Launch it once and wait for the whale icon in the system tray to stop
   animating. `docker` commands fail until the engine is up.
3. In **PowerShell** (or Command Prompt — `docker` is on the `PATH`):

   ```powershell
   git clone https://github.com/endre90/artwork_ejas.git
   cd artwork_ejas
   docker compose up
   ```

   No Git installed? Use *Code → Download ZIP* on the GitHub page, extract it,
   and `cd` into the folder instead.
4. Open <http://localhost:8080> in any browser on Windows.

**When something goes wrong**

| Symptom | Cause and fix |
| --- | --- |
| `error during connect` / `the system cannot find the file specified` | The engine is not running. Start Docker Desktop and wait for the tray icon to settle. |
| Docker Desktop won't start, mentioning virtualization or WSL | Enable Intel VT-x / AMD-V in the BIOS/UEFI; then `wsl --update` in an administrator PowerShell. |
| `Bind for 127.0.0.1:8080 failed: port is already allocated` | Something else holds :8080 (`netstat -ano \| findstr :8080`). Change the mapping in `docker-compose.yml` to e.g. `127.0.0.1:8081:8080` and open :8081. |
| `failed to fetch` or TLS errors while downloading crates | A corporate proxy or TLS-inspecting antivirus. Enter the proxy under Docker Desktop → *Settings* → *Resources* → *Proxies*. |
| `docker: 'compose' is not a docker command` | A Docker Desktop old enough to predate Compose v2. Update it, or run `docker-compose up` (hyphenated) instead. |

### Run natively (for development)

**Prerequisites**

1. Rust 1.96 or newer: <https://www.rust-lang.org/tools/install>
2. Z3 **from your package manager, not from source**:
   - Debian/Ubuntu: `sudo apt install libz3-dev libclang-dev`
   - macOS: `brew install z3`
   - Fedora: `sudo dnf install z3-devel clang-devel`

   The `-dev` package matters: `z3-sys` needs both `libz3.so` *and* the `z3.h`
   headers, because it regenerates its FFI bindings with bindgen at build time
   (which is also why `libclang` is required). If Z3 lives somewhere
   non-standard, point the build at it with
   `export Z3_SYS_Z3_HEADER=/your/prefix/include/z3.h`.

**Running**

Two terminals:

```
cargo run -p ejas-server          # the solver, on :8080
cargo run -p ejas-ui              # the same UI, as a desktop window
```

The desktop build talks to the same HTTP API as the browser build, so there is
one UI codebase rather than two. It reads `EJAS_SERVER` for the server address
and defaults to `http://127.0.0.1:8080`.

To work on the web build itself, use [trunk](https://trunkrs.dev):

```
cargo install --locked trunk
trunk build --release             # writes crates/ejas-server/dist/
cargo run -p ejas-server          # serves that bundle at :8080
```

`trunk serve` also works for live reload; it proxies `/api` to a server you
start separately on :8080.

**Tests**

```
cargo test -p ejas-server         # weight presets behave as advertised
cargo test -p artwork_ejas        # the solver's own evaluation harnesses
```

## Usage

### Getting your data in

Three ways, offered on the first screen:

- **Load a matrix file** — a JSON file describing the station. You can also
  drag it onto the window.
- **Set up from scratch** — define the jobs, then add employees with their
  competences and preferences. Export the result as a matrix file to reuse it
  tomorrow. The roster says who works here and what they can do; it does not
  say who leads.
- **Try an example** — the anonymised VCE and GTO rosters bundled with the
  repository.

Optionally load an **assignment history** as well. History is what the fairness
term works on: without it the solver has no way to know who did what yesterday,
and cannot rotate work. Both the day-based (`{"day": ...}`) and pass-based
(`{"pass": ...}`) file formats are accepted.

### Today's team leader

Team leaders rotate, so the leader is picked on the **Today** tab, not on the
roster — pick them from *Team leader today*, or click the `TL` chip next to a
name. The choice travels with each solve and overrides whatever `role` the
loaded matrix file carried, so the same roster can be solved for a different
leader every day without editing it.

The leader is who the `alpha` penalty protects: the solver keeps them off a job
unless the alternative is worse. Solving is blocked until one is chosen, since
with nobody leading the `alpha` term never fires and the roster is just a pool
of operators.

### Matrix file format

```json
{
  "stations": {
    "CE": {
      "ergo_score": { "O1": 1, "O2": 3 },
      "people": [
        {
          "name": "A",
          "role": "Operator",
          "competences": ["O1", "O2"],
          "preferences": ["O2"]
        },
        {
          "name": "B",
          "role": "TeamLeader",
          "competences": ["O1"],
          "preferences": []
        }
      ]
    }
  }
}
```

- `ergo_score` maps each job to how physically demanding it is; its keys define
  the station's job list.
- `preferences` is **ordered**: position in the list is the preference rank, so
  the first entry is the most wanted job.
- `role` is accepted but is **not** what decides who leads. Leaders rotate, so
  today's is chosen on the Today tab and sent with each solve; a `TeamLeader`
  in the file is only used as the initial suggestion when you load it. A file
  where everyone is an `Operator` is perfectly valid.
- `data/factory/VCE_matrix.json` and `data/factory/GTO_matrix.json` are working
  examples.

### Weights

The objective is

```
offset + omega * preference - alpha * leader - beta * external - gamma * historical
```

where the historical term is, per assigned employee-job pair, the number of
times that pair occurred in the last `tau` days, multiplied by the job's
ergonomic multiplier `E_max - E_j + 1` so that repeating a physically hard job
costs more than repeating an easy one.

Rather than tuning the weights, pick a strategy — or choose **Manual** and edit
every weight yourself. Manual starts from whichever strategy you had selected,
so you tune from a working baseline.

**The weights are derived from the station, not hardcoded.** Every threshold
below is a statement about outweighing some *other* term of the objective, and
those terms scale with the station: a `gamma` that dominates preferences in an
8-job station is a rounding error in a 40-job one. The formulas are written in
terms of

| Symbol | Meaning |
|---|---|
| `N`, `M` | operators (leader included) and jobs in the station |
| `E_min`, `E_max` | worst and best ergonomic score in the station |
| `tau` | days of history the fairness term looks back over |
| `d_limit` | consecutive days on one job after which rotating off is mandatory |
| `K` | how many times more undesirable an external operator is than the leader |

`N`, `M`, `E_min` and `E_max` are read off the roster. `d_limit`, `K` and `tau`
are policy, shown under the strategy and editable; they default to 2, 2 and 8.

**Balanced** negotiates between stated preferences and compounding fatigue.
Preferences are the anchor, and the ergonomic penalty is sized so that after
`d_limit` consecutive days on a job, rotating off it beats any preference gain
available:

```
omega = 1        gamma > omega * M * N / (d_limit * E_min)
alpha > N * M * omega                      beta = K * alpha
```

**Safety-first** minimises physical strain. The ergonomic penalty becomes the
anchor (`gamma = 1`) and preferences fall to tie-breakers between equally safe
assignments (`omega = 0.01`). The leader and external penalties must now clear
the largest ergonomic penalty the station can accumulate over a whole window:

```
gamma = 1        omega = 0.01
alpha > N * tau * E_max                    beta = K * alpha
```

**Happiness-first** suits stations whose jobs are physically alike. It removes
the ergonomic multiplier from the objective entirely, so `gamma` becomes a pure
boredom penalty that still has to outgrow the reward for repeating a favourite
job. `alpha` and `beta` are Balanced's:

```
omega = 1        gamma > omega * M * N / (d_limit * E_max)
```

Since the weights must be integers, Safety-first's `omega = 0.01, gamma = 1` is
applied as `omega = 1, gamma = 100`, and its `alpha` bound is scaled by the
same 100. Scaling every term leaves the argmax untouched. `offset` is derived
too — it bounds the total penalty so the reported score stays positive, and is
cosmetic.

What that works out to on the two bundled rosters, with the default `d_limit`,
`K` and `tau`:

| Station | Strategy | omega | alpha | beta | gamma | ergo multiplier |
|---|---|---|---|---|---|---|
| VCE (N=16, M=12, E 1..4) | Balanced | 1 | 193 | 386 | 97 | on |
| | Safety-first | 1 | 51201 | 102402 | 100 | on |
| | Happiness-first | 1 | 193 | 386 | 25 | **off** |
| GTO (N=11, M=8, E 1..3) | Balanced | 1 | 89 | 178 | 45 | on |
| | Safety-first | 1 | 26401 | 52802 | 100 | on |
| | Happiness-first | 1 | 89 | 178 | 15 | **off** |

`cargo test -p ejas-server` checks both the formulas and the behaviour they
produce. Two results there are worth knowing before you pick a strategy:

- **The strategies only separate on a constrained roster.** VCE can satisfy
  everyone with no repeats, no leader and no externals, so all three land on
  the same assignment. GTO is where they diverge.
- **On GTO, Safety-first ends up with *more* ergonomic strain than Balanced**
  (2 against 0). Sizing `alpha` above the whole ergonomic range is what does
  it: the only way to clear the last hard repeat is to put the team leader on
  the job, and Safety-first will not spend the leader for that, while Balanced
  — whose `alpha` only has to clear the preference swing — will. If minimising
  strain is meant to outrank protecting the leader, `alpha` is the term to
  revisit.

One formula is implemented as specified but looks inconsistent, and is flagged
here rather than silently changed: Happiness-first divides by `E_max` where
Balanced divides by `E_min`. Dropping the multiplier *shrinks* the historical
penalty, so a rotation guarantee on its own would call for a larger `gamma`
there, not a smaller one; the two agree only when `E_min == E_max`. If the
intent was to swap those denominators, `WeightPreset::params` is the one place
to change.

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

- $N$: Number of employees
- $M$: Number of jobs
- $x_{ij}$: Binary decision vaiable representing operator $i$ performing job $j$
- $C_{ij}$: Binary competence matrix where $C_{ij} = 1$ if employee $i$ can perform job $j$, and $C_{ij} = 0$ otherwise
- $P_{ij}$: Preference rank matrix where $P_{ij}$ is the preference rank of job $j$ for employee $i$. Lower values (taken as index) in $P_{ij}$ indicate higher preference
- $k$: Index of a team leader, if it exists.
- $\alpha$: How strongly to discourage the use of a team leader. The team leader is allocating operators and doing other work, so performing operations 
should only be done when there is not enough competent operators in the station. 

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
\text{Maximize} \quad \sum_{i=1}^{N} \sum_{j=1}^{M} (M - P_{ij}) \cdot x_{ij} - \alpha \sum_{j=1}^{M} x_{kj} 
$$

### Constraints

1. Each employee is assigned to at most one job:

$$
\sum_{j=1}^{M} x_{ij} \leq 1 \quad \forall i \in N
$$

2. Each job is assigned exactly one employee:

$$
\sum_{i=1}^{N} x_{ij} = 1 \quad \forall j \in \{1, \ldots, M\}
$$

3. Each job must be covered by at least one employee who is competent to perform it:

$$
\sum_{i=1}^{N} C_{ij} \cdot x_{ij} \geq 1 \quad \forall j \in M
$$

4. Only assign jobs to competent employees (implication is used because the employee might be competent to perform the job, but doesn't have to be allocated to perform it):

$$
x_{ij} \implies C_{ij} \quad \forall i \in N, \forall j \in M
$$


## Step 2: External assignment
In this step, the static assignment problem is extended with the possibility to have more jobs than employees, or to have jobs for which not enough competences exist. In this case, external employees can be allocated to complete the job, however this has a high cost and should be avoided if possible.

### Notation

- $N$: Number of employees
- $M$: Number of jobs
- $x_{ij}$: Binary decision vaiable representing operator $i$ performing job $j$
- $C_{ij}$: Binary competence matrix where $C_{ij} = 1$ if employee $i$ can perform job $j$, and $C_{ij} = 0$ otherwise
- $P_{ij}$: Preference rank matrix where $P_{ij}$ is the preference rank of job $j$ for employee $i$. Lower values (taken as index) in $P_{ij}$ indicate higher preference
- $k$: Index of a team leader, if it exists.
- $\alpha$: How strongly to discourage the use of a team leader. The team leader is allocating operators and doing other work, so performing operations 
should only be done when there is not enough competent operators in the station. 
- $\beta$: How strongly to discourage the use of external employees. If there is not enough external employees in the station, external employees are called in to complete the jobs.
- NOTE: As a general rule, if the team leader is competent to perform an operation for which a competence doesn't exist, or there are not enough employees, the team leader should always perform the job before calling in external employees. This can be tweaked by changing the $\alpha$ and $\beta$ weights.

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
\text{Maximize} \quad \sum_{i=1}^{N} \sum_{j=1}^{M} (M - P_{ij}) \cdot x_{ij} - \alpha \sum_{j=1}^{M} x_{kj} - \beta \sum_{j=1}^{M} e_j
$$

where $\beta$ is a penalty factor for using external employees.

### Constraints

1. Each internal employee is assigned to at most one job:

$$
\sum_{j=1}^{M} x_{ij} \leq 1 \quad \forall i \in \{1, \ldots, N\}
$$

2. Each job is assigned exactly to either one internal employee or one external employee:

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


## Step 3: Fair assignment with historic data

This model ensures that employees are fairly assigned a job for the current day, taking into account their preferences and competences while considering historical data to avoid over-assigning the same jobs to the same employees repeatedly. The objective function balances maximizing the total preference score, minimizing the use of external employees, and promoting fairness in job assignments.

### Notation

- $N$: Number of employees
- $M$: Number of jobs
- $x_{ij}$: Binary decision vaiable representing operator $i$ performing job $j$
- $C_{ij}$: Binary competence matrix where $C_{ij} = 1$ if employee $i$ can perform job $j$, and $C_{ij} = 0$ otherwise
- $P_{ij}$: Preference rank matrix where $P_{ij}$ is the preference rank of job $j$ for employee $i$. Lower values (taken as index) in $P_{ij}$ indicate higher preference
- $k$: Index of a team leader, if it exists.
- $\alpha$: How strongly to discourage the use of a team leader. The team leader is allocating operators and doing other work, so performing operations 
should only be done when there is not enough competent operators in the station. 
- $\beta$: How strongly to discourage the use of external employees. If there is not enough external employees in the station, external employees are called in to complete the jobs.
- $\tau$: Number of days to consider in the historical data (from last day to last day - $\tau$).
- $\gamma$: A weight controlling how strongly to penalize assigning the same employee–job pair that was frequently assigned in the past $\tau$ days.
- $\sigma$: Optional assignment threshold, only necessary if there is an absolute limit (e.g., ergonomic score or union rules).
- $H_{ij}(\tau)$: Historical count matrix where $H_{ij}(\tau)$ is the number of times worker $i$ has performed job $j$ in the past $\tau$ days.
- NOTE: As a general rule, if the team leader is competent to perform an operation for which a competence doesn't exist, or there are not enough employees, the team leader should always perform the job before calling in external employees. This can be tweaked by changing the $\alpha$ and $\beta$ weights.

### Decision Variables

- $x_{ij}$: Binary decision variable such that:

$$
x_{ij} =
\begin{cases} 
1 & \text{if worker } i \text{ is assigned to job } j \\ 
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

- $\lambda$: Penalty factor for using external employees.
- $\beta$: Weight for the fairness term.
- $\alpha$: Maximum allowed number of assignments (including historical data) for any job.

### Constraints

1. Each internal worker is assigned to at most one job:

$$
\sum_{j=1}^{M} x_{ij} \leq 1 \quad \forall i \in \{1, \ldots, N\}
$$

2. Each job is assigned to either one internal worker or one external worker:

$$
\sum_{i=1}^{N} x_{ij} + e_j = 1 \quad \forall j \in \{1, \ldots, M\}
$$

3. Only assign jobs to employees who are competent to perform them:

$$
x_{ij} \leq C_{ij} \quad \forall i, j
$$

4. Each job must be covered by at least one competent internal worker or an external worker:

$$
\sum_{i=1}^{N} C_{ij} \cdot x_{ij} + e_j \geq 1 \quad \forall j \in \{1, \ldots, M\}
$$

5. OPTIONAL: Strictly limit the maximum number of assignments (but this is a hard constraint rather than a fairness preference. Usually, this is only necessary if there is an absolute limit (e.g., ergonomic score or union rules)):

$$
H_{ij}(\tau) + x_{ij} \leq \sigma \quad \forall i \in \{1, \ldots, N\}, \forall j \in \{1, \ldots, M\}
$$

### Objective Function

The objective function can be expressed as:

$$
\text{Maximize} 
\sum_{i=1}^{N} \sum_{j=1}^{M} \bigl(M - P_{i,j}\bigr)\,x_{ij}
\;-\;
\alpha \sum_{j=1}^{M} x_{kj}
\;-\;
\beta \sum_{j=1}^{M} e_{j}
\;-\;
\gamma \sum_{i=1}^{N} \sum_{j=1}^{M} H_{i,j}(\tau)\, x_{ij}.
$$

Where:


- $ \sum_{i=1}^{N} \sum_{j=1}^{M} (M - P_{ij}) \, x_{ij}$ maximizes total preference, since a lower preference rank $ P_{ij} $ yields a higher $ (M - P_{ij}) $.  
- $ \alpha \sum_{j=1}^{M} x_{k,j} $ penalizes using the team leader (employee \(k\)).  
- $ \beta \sum_{j=1}^{M} e_{j} $ penalizes using external employees.  
- $ \gamma \sum_{i=1}^{N} \sum_{j=1}^{M} H_{ij}(\tau)\, x_{ij} $ adds a fainess penalty for assigning a job $j$ to an employee $i$ who has in the last $\tau$ days done that job many times, promoting a more balanced distribution of tasks.

## Step 4: Fair assignment with historic data and ergonomics

This model ensures that employees are fairly assigned a job for the current day, taking into account their preferences and competences while considering historical data to avoid over-assigning the same jobs to the same employees repeatedly. In this model, the operation ergonomics are also considered. The objective function balances maximizing the total preference score, minimizing the use of external employees, and promoting fairness in job assignments.

### Notation

- $N$: Number of employees
- $M$: Number of jobs
- $x_{ij}$: Binary decision vaiable representing operator $i$ performing job $j$
- $C_{ij}$: Binary competence matrix where $C_{ij} = 1$ if employee $i$ can perform job $j$, and $C_{ij} = 0$ otherwise
- $P_{ij}$: Preference rank matrix where $P_{ij}$ is the preference rank of job $j$ for employee $i$. Lower values (taken as index) in $P_{ij}$ indicate higher preference
- $k$: Index of a team leader, if it exists.
- $\alpha$: How strongly to discourage the use of a team leader. The team leader is allocating operators and doing other work, so performing operations 
should only be done when there is not enough competent operators in the station. 
- $\beta$: How strongly to discourage the use of external employees. If there is not enough external employees in the station, external employees are called in to complete the jobs.
- $\tau$: Number of days to consider in the historical data (from last day to last day - $\tau$).
- $\gamma$: A weight controlling how strongly to penalize assigning the same employee–job pair that was frequently assigned in the past $\tau$ days.
- $\sigma$: Optional assignment threshold, only necessary if there is an absolute limit (e.g., ergonomic score or union rules).
- $H_{ij}(\tau)$: Historical count matrix where $H_{ij}(\tau)$ is the number of times worker $i$ has performed job $j$ in the past $\tau$ days.
- $\theta \geq 0$ is a parameter controlling how much the historical count erodes the ergonomics benefit.
- $\delta$ controls how much weight is given to the ergonomics score in the objective function. A higher $\delta means ergonomics will play a larger role in determining the optimal assignments, prioritizing jobs with better adjusted ergonomics scores $E^\mathrm{eff}_{ij}(\tau)$. A lower $\delta$ reduces the influence of ergonomics, allowing other factors like preferences or fairness to dominate. 

### Decision Variables

- $x_{ij}$: Binary decision variable such that:

$$
x_{ij} =
\begin{cases} 
1 & \text{if worker } i \text{ is assigned to job } j \\ 
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

- $\lambda$: Penalty factor for using external employees.
- $\beta$: Weight for the fairness term.
- $\alpha$: Maximum allowed number of assignments (including historical data) for any job.

### Constraints

1. Each internal worker is assigned to at most one job:

$$
\sum_{j=1}^{M} x_{ij} \leq 1 \quad \forall i \in \{1, \ldots, N\}
$$

2. Each job is assigned to either one internal worker or one external worker:

$$
\sum_{i=1}^{N} x_{ij} + e_j = 1 \quad \forall j \in \{1, \ldots, M\}
$$

3. Only assign jobs to employees who are competent to perform them:

$$
x_{ij} \leq C_{ij} \quad \forall i, j
$$

4. Each job must be covered by at least one competent internal worker or an external worker:

$$
\sum_{i=1}^{N} C_{ij} \cdot x_{ij} + e_j \geq 1 \quad \forall j \in \{1, \ldots, M\}
$$

5. OPTIONAL: Strictly limit the maximum number of assignments (but this is a hard constraint rather than a fairness preference. Usually, this is only necessary if there is an absolute limit (e.g., ergonomic score or union rules)):

$$
H_{ij}(\tau) + x_{ij} \leq \sigma \quad \forall i \in \{1, \ldots, N\}, \forall j \in \{1, \ldots, M\}
$$

### Objective Function

The objective function can be expressed as:

$$
\text{Maximize} \quad
\sum_{i=1}^{N} \sum_{j=1}^{M} \bigl(M - P_{ij}\bigr) x_{ij} \\
\;-\; \alpha \sum_{j=1}^{M} x_{kj} \\
\;-\; \beta \sum_{j=1}^{M} e_{j} \\
\;-\; \gamma \sum_{i=1}^{N} \sum_{j=1}^{M} H_{ij}(\tau) x_{ij} \\
\;+\; \delta \sum_{i=1}^{N} \sum_{j=1}^{M} E^\mathrm{eff}_{ij}(\tau) x_{ij}.
$$

Where:


- $ \sum_{i=1}^{N} \sum_{j=1}^{M} (M - P_{ij}) x_{ij}$ maximizes total preference, since a lower preference rank $ P_{ij} $ yields a higher $ (M - P_{ij}) $.  
- $ \alpha \sum_{j=1}^{M} x_{kj} $ penalizes using the team leader (employee \(k\)).  
- $ \beta \sum_{j=1}^{M} e_{j} $ penalizes using external employees.  
- $ \gamma \sum_{i=1}^{N} \sum_{j=1}^{M} H_{ij}(\tau) x_{ij} $ adds a fainess penalty for assigning a job $j$ to an employee $i$ who has in the last $\tau$ days done that job many times, promoting a more balanced distribution of tasks.
- $ E^\mathrm{eff}_{ij}(\tau) = \frac{E_{j}}{1 + \theta H_{ij}(\tau)} $

**How the Ergonomics Works**

- $E_j$ is the base (intrinsic) ergonomics score of job $j$.  
- $H_{ij}(\tau)$ is the historical assignment count for how many times employee $i$ has performed job $j$ in the last $\tau$ days.  
- $\theta \geq 0$ is a parameter controlling how strongly the historical count reduces the base ergonomics.  
- As $H_{ij}(\tau)$ increases, the denominator $1 + \theta H_{ij}(\tau)$ grows, thereby lowering $E^\mathrm{eff}_{ij}(\tau)$. This discourages repeatedly assigning the same employee to the same job from an ergonomics perspective.



## Step 5: Fair assignment with horizon

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
\Gamma = \beta \sum_{i=1}^{N} abs\left(\sum_{j=1}^{M} \sum_{t=1}^{T} \left( P_{ij} \leq \sigma \right) \cdot x_{ijt} - \frac{T \cdot \sigma}{N} \right) , \quad \sigma=\left\lfloor \frac{k \cdot M}{100} \right\rfloor
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
x_{ijt} \implies C_{ij} \quad \forall i, j, t
$$

4. Each job must be covered by at least one competent internal worker or an external worker:

$$
\sum_{i=1}^{N} C_{ij} \cdot x_{ijt} + e_{jt} \geq 1 \quad \forall j \in \{1, \ldots, M\}, \forall t \in \{1, \ldots, T\}
$$



