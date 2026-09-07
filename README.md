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