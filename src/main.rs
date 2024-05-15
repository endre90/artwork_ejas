use artwork_ejas::calculate_static_assignment;
use z3::{
    ast::{Bool, Int},
    *,
};

// use crate::*;

fn main() {
    // Number of workers and jobs
    let n = 3;
    let m = 3;

    // Example competence matrix (binary)
    let c: Vec<Vec<i32>> = vec![
        vec![1, 0, 1], // Worker 0 can perform jobs 0 and 2
        vec![1, 1, 0], // Worker 1 can perform jobs 0 and 1
        vec![0, 1, 1], // Worker 2 can perform jobs 1 and 2
    ];

    // Example preferences list
    let p = vec![
        vec![2, 0, 1], // Worker 0 prefers job 2, then 0, then 1
        vec![0, 1, 2], // Worker 1 prefers job 0, then 1, then 2
        vec![1, 2, 0], // Worker 2 prefers job 1, then 2, then 0
    ];

    // Create the Z3 context and optimizer
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let optimizer = Optimize::new(&ctx);

    // Create boolean variables for assignments
    let x: Vec<Vec<Bool>> = (0..n)
        .map(|i| {
            (0..m)
                .map(|j| Bool::new_const(&ctx, format!("x_{}_{}", i, j)))
                .collect()
        })
        .collect();

    // Constraints: Each worker is assigned at most one job
    for i in 0..n {
        let worker_constraints: Vec<_> = (0..m).map(|j| x[i][j].clone()).collect();
        let at_most_one_job_per_worker = ast::Bool::pb_eq(
            &ctx,
            worker_constraints
                .iter()
                .map(|x| (x, 1))
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1,
        );
        // pb_eq and soft instead
        optimizer.assert_soft(&at_most_one_job_per_worker, 1, None);

        // trying pb_le and hard, why doesn't this work?
        // optimizer.assert(&at_most_one_job_per_worker);
    }

    // Constraints: Each job is assigned to at most one worker
    for j in 0..m {
        let job_constraints: Vec<_> = (0..n).map(|i| x[i][j].clone()).collect();
        let at_most_one_worker_per_job = ast::Bool::pb_eq(
            &ctx,
            job_constraints
                .iter()
                .map(|x| (x, 1))
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1,
        );
        // pb_eq and soft instead
        optimizer.assert_soft(&at_most_one_worker_per_job, 1, None);

        // trying pb_le and hard, why doesn't this work?
        // optimizer.assert(&at_most_one_worker_per_job);
    }

    // Constraints: Only assign jobs that workers are competent to perform
    for i in 0..n {
        for j in 0..m {
            optimizer.assert_soft(
                &Bool::implies(&x[i][j], &Bool::from_bool(&ctx, c[i][j] == 1)),
                1,
                None,
            );
        }
    }

    // Objective: Maximize preferences
    let mut preference_score = Vec::new();
    for i in 0..n {
        for (rank, &j) in p[i].iter().enumerate() {
            let score = (m - rank) as i32;
            preference_score.push((Bool::implies(&x[i][j], &Bool::from_bool(&ctx, true)), score));
        }
    }

    let preference_score_sum: Vec<_> = preference_score
        .iter()
        .map(|(b, s)| {
            b.ite(
                &z3::ast::Int::from_i64(&ctx, *s as i64),
                &z3::ast::Int::from_i64(&ctx, 0),
            )
        })
        .collect();
    optimizer.maximize(&Int::add(&ctx, &preference_score_sum));

    // Check satisfiability and print the solution
    match optimizer.check(&[]) {
        SatResult::Sat => {
            let model = optimizer.get_model().unwrap();
            let mut assignment = Vec::new();
            let mut total_score = 0;

            for i in 0..n {
                for j in 0..m {
                    if model.eval(&x[i][j], true).unwrap().as_bool().unwrap() {
                        assignment.push((i, j));
                        total_score += m - p[i].iter().position(|&x| x == j).unwrap();
                    }
                }
            }

            println!("Optimal assignment: {:?}", assignment);
            println!("Total preference score: {}", total_score);
        }
        SatResult::Unsat => println!("No solution found"),
        _ => println!("Solver failed"),
    }

    let employees = vec!("a", "b", "c", "d", "e").iter().map(|x| x.to_string()).collect();
    let jobs = vec!("1", "2", "3", "4", "5").iter().map(|x| x.to_string()).collect();
    let competences = vec!(
        ("a".to_string(), vec!("1", "3").iter().map(|x| x.to_string()).collect()),
        ("b".to_string(), vec!("2").iter().map(|x| x.to_string()).collect()),
        ("c".to_string(), vec!("3", "4", "5").iter().map(|x| x.to_string()).collect()),
        ("d".to_string(), vec!("3", "5").iter().map(|x| x.to_string()).collect()),
        ("e".to_string(), vec!("4", "1", "2", "3").iter().map(|x| x.to_string()).collect())
    );
    let preferences = vec!(
        ("a".to_string(), vec!("1").iter().map(|x| x.to_string()).collect()),
        ("b".to_string(), vec!("2").iter().map(|x| x.to_string()).collect()),
        ("c".to_string(), vec!("3", "4").iter().map(|x| x.to_string()).collect()),
        ("d".to_string(), vec!("3").iter().map(|x| x.to_string()).collect()),
        ("e".to_string(), vec!("4", "1").iter().map(|x| x.to_string()).collect())
    );
    let _solution = calculate_static_assignment(&employees, &jobs, &competences, &preferences);

}
