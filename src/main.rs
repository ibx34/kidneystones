use std::{
    cmp::max,
    collections::HashMap,
    path::{Path, PathBuf},
};

use git2::{BranchType::Local, DiffOptions, Oid, Repository, Tree};

use crate::models::RepoFile;

mod app;
mod config;
mod db;
mod models;
mod routes;
mod web;

// use crate::{
//     app::App,
//     routes::{
//         git::{info_refs, recieve_pack, upload_pack},
//         repos::{create_repo, get_repo, get_repo_tree},
//     },
//     web::register_web_routes,
// };

fn get_file_or_tree_owner(repo: &Repository, files: Vec<(Oid, RepoFile)>) {
    // filename: (Parent, File)
    let mut parents_and_chidlren: HashMap<&str, (Oid, Oid)> = HashMap::new();
    for file in &files {
        let mut revwalk = repo.revwalk().unwrap();
        revwalk.set_sorting(git2::Sort::TIME).unwrap();
        revwalk.push_head().unwrap();

        for commit_id in revwalk {
            let commit_id = commit_id.unwrap();
            // println!("LOOKING AT COMMIT {commit_id:?}\n\n");
            // println!("Checking Commit: {:?}", commit_id);
            // if commits_to_check.contains(&commit_id) {
            //     println!("Tree contains: {:?}", commit_id);
            let commit = repo.find_commit(commit_id).unwrap();
            let prev_tree = if commit.parent_count() > 0 {
                let prev_commit = commit.parent(0).unwrap();
                Some(prev_commit.tree().unwrap())
            } else {
                None
            };

            let tree = commit.tree().unwrap();
            // let prev_tree = prev_commit.tree().unwrap();

            let mut opts = DiffOptions::new();
            let diff = repo
                .diff_tree_to_tree(prev_tree.as_ref(), Some(&tree), Some(&mut opts))
                .unwrap();

            let mut deltas = diff.deltas();

            let file_name = file.1.filename.clone();
            let contains = deltas.any(|dd| {
                let new_file_path = dd.new_file().path().unwrap();
                // File || Dir
                new_file_path.eq(Path::new(&file_name)) || new_file_path.starts_with(&file_name)
            });

            if contains {
                if parents_and_chidlren.get(file.1.filename.as_str()).is_none() {
                    parents_and_chidlren.insert(&file.1.filename, (commit.id(), file.0));
                }
                // println!(
                //     "Parent of file \"{:?}\" ({:?}) is {:?}",
                //     file_path.0,
                //     file_path.1,
                //     commit.id()
                // );
            }
        }
    }
    println!("{parents_and_chidlren:#?}");
    //     // Ignore merge commits (2+ parents) because that's what 'git whatchanged' does.
    //     // Ignore commit with 0 parents (initial commit) because there's nothing to diff against
    //     if commit.parent_count() == 1 {
    //         let prev_commit = commit.parent(0).unwrap();
    //         println!("{:?} belongs to {:?}", commit_id, prev_commit.id());
    //         // let tree = commit.tree().unwrap();
    //         // let prev_tree = prev_commit.tree().unwrap();
    //         // let diff = repo
    //         //     .diff_tree_to_tree(Some(&prev_tree), Some(&tree), None)
    //         //     .unwrap();
    //         // for delta in diff.deltas() {
    //         //     let file_path = delta.new_file().path().unwrap();
    //         //     let file_mod_time = commit.time();
    //         //     let unix_time = file_mod_time.seconds();
    //         //     mtimes
    //         //         .entry(file_path.to_owned())
    //         //         .and_modify(|t| *t = max(*t, unix_time))
    //         //         .or_insert(unix_time);
    //         // }
    //     }
    // }
}

fn main() {
    let git_repo = match Repository::open_bare("./tests/ibx34/why-no-work") {
        Ok(repo) => repo,
        Err(err) => {
            panic!("{err:?}");
        }
    };
    let requested_branch = git_repo.find_branch("master", Local).unwrap();

    let _tree = requested_branch.get().peel_to_tree().unwrap();

    let tree = _tree
        .iter()
        .map_while(|e: git2::TreeEntry<'_>| {
            let file = e.to_object(&git_repo).unwrap();
            if let Some(blob) = file.as_blob() {
                println!("1: {:?}", file.kind());

                println!("Name: {:?}", e.name());
                return Some((
                    blob.id(),
                    RepoFile {
                        filename: e.name().unwrap().to_string(),
                        hash: file.id().to_string(),
                        size: blob.content().to_vec().len(),
                    },
                ));
            }
            None
        })
        .collect::<Vec<(Oid, RepoFile)>>();

    get_file_or_tree_owner(&git_repo, tree);
}

// use anyhow::Result;
// use axum::{
//     extract::Path,
//     response::{Html, IntoResponse},
//     // middleware::from_fn_with_state,
//     routing::{get, get_service, post},
//     Router,
// };
// use axum_extra::body::AsyncReadBody;
// use reqwest::StatusCode;
// use std::net::SocketAddr;
// use tower_http::{services::ServeDir, trace::TraceLayer};

// async fn web() -> impl IntoResponse {
//     let Ok(file) = tokio::fs::File::open("/home/alfredo/kidney-stones/web/dist/index.html").await else {
//         panic!();
//     };
//     (AsyncReadBody::new(file))
// }

// #[tokio::main]
// async fn main() -> Result<()> {
//     tracing_subscriber::fmt()
//         .with_max_level(tracing::Level::INFO)
//         .pretty()
//         .init();

//     let app = App::init().await?;
//     // throw the result away, we dont care.
//     let _ = app.create_account("ibx34", "haha").await;

//     let repositories = Router::new()
//         .route("/create", post(create_repo))
//         .route("/:owner/:name", get(get_repo))
//         .route("/:owner/:name/tree/:branch", get(get_repo_tree));
//     let api = Router::new()
//         .route("/:user/:repo/git-upload-pack", post(upload_pack))
//         .route("/:user/:repo/git-receive-pack", post(recieve_pack))
//         .route("/:user/:repo/info/refs", get(info_refs))
//         .nest("/repos", repositories);
//     //.layer(from_fn_with_state(app_state.clone(), session_middleware));
//     let web_routes = register_web_routes();
//     let app = Router::new()
//         .nest_service(
//             "/assets",
//             get_service(ServeDir::new("/home/alfredo/kidney-stones/assets")),
//         )
//         .nest_service(
//             "/dist",
//             get_service(ServeDir::new("/home/alfredo/kidney-stones/web/dist/")),
//         )
//         // git remote add <name> http://127.0.0.1:8080/ibx34/<repo-name>[.git]
//         .nest("/api", api)
//         .nest("/", web_routes)
//         .layer(TraceLayer::new_for_http())
//         .with_state(app);

//     let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
//     println!("Starting on: {addr:?}");
//     axum::Server::bind(&addr)
//         .serve(app.into_make_service_with_connect_info::<SocketAddr>())
//         .await?;
//     println!("Started");
//     Ok(())
// }
