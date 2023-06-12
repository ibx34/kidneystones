use git2::{
    Blob, BranchType::Local, DescribeOptions, DiffDelta, DiffOptions, Oid, Repository, Tree,
};

mod app;
mod config;
mod db;
mod models;
mod routes;
mod web;

use crate::{
    app::App,
    models::RepoFile,
    routes::{
        git::{info_refs, recieve_pack, upload_pack},
        repos::{create_repo, get_repo, get_repo_tree},
    },
    web::register_web_routes,
};

// fn get_file_or_tree_latest_commit(
//     repo: &Repository,
//     commit: Option<Oid>,
//     file: BlobOrTree<'_>,
// ) -> Option<(String, Oid)> {
//     let mut revwalk = repo.revwalk().unwrap();
//     revwalk.set_sorting(git2::Sort::TIME).unwrap();
//     revwalk.push_head().unwrap();
//     if let Some(oid) = commit {
//         revwalk.push(oid).unwrap();
//     } else {
//         revwalk.push_head().unwrap();
//     }
//     for commit_id in revwalk {
//         let commit_id = commit_id.unwrap();

//         let commit = repo.find_commit(commit_id).unwrap();
//         let prev_tree = if let Ok(prev_commit) = commit.parent(0) {
//             Some(prev_commit.tree().unwrap())
//         } else {
//             None
//         };
//         let tree = commit.tree().unwrap();

//         let diff = repo
//             .diff_tree_to_tree(prev_tree.as_ref(), Some(&tree), None)
//             .unwrap();

//         let mut deltas = diff.deltas();

//         let contains = deltas.any(|dd| {
//             let new_file = dd.new_file();
//             let new_file_path = new_file.path().unwrap();
//             new_file_path.eq(Path::new(&file.name))
//                 || new_file_path.starts_with(&file.name)
//                 || file.__inner.id() == new_file.id()
//         });
//         if contains {
//             return Some((file.name.to_string(), commit.id()));
//         }
//     }

//     None
// }

use anyhow::Result;
use axum::{
    response::{Html, IntoResponse},
    // middleware::from_fn_with_state,
    routing::{get, get_service, post},
    Router,
};
use axum_extra::body::AsyncReadBody;
use reqwest::StatusCode;
use std::{
    boxed::Box,
    cmp::max,
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tower_http::{services::ServeDir, trace::TraceLayer};

// #[derive(Debug, Clone)]
// pub enum BlobOrTree2<'a> {
//     Blob(Blob<'a>),
//     Tree(Vec<Box<BlobOrTree<'a>>>),
// }

// impl<'a> BlobOrTree2<'a> {
//     pub fn id(&self) -> Oid {
//         match self {
//             BlobOrTree2::Blob(b) => b.id(),
//             BlobOrTree2::Tree(_) => unreachable!(),
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct BlobOrTree<'a> {
//     pub __inner: BlobOrTree2<'a>,
//     pub name: String,
// }

// fn handle_tree<'a>(repo: &'a Repository, tree: Tree<'a>) -> Vec<BlobOrTree<'a>> {
//     tree.into_iter()
//         .map_while(|tree_entry| {
//             let git_obj = tree_entry.to_object(&repo).unwrap();

//             match git_obj.kind() {
//                 Some(git2::ObjectType::Blob) => {
//                     let b = git_obj.as_blob()?;
//                     Some(BlobOrTree {
//                         __inner: BlobOrTree2::Blob(b.clone()),
//                         name: tree_entry.name().unwrap_or("").to_string(),
//                     })
//                 }
//                 Some(git2::ObjectType::Tree) => {
//                     let tree = git_obj.as_tree()?;
//                     let boxed_tree = handle_tree(&repo, tree.clone())
//                         .into_iter()
//                         .map(|e| Box::new(e))
//                         .collect::<Vec<Box<BlobOrTree<'_>>>>();
//                     Some(BlobOrTree {
//                         __inner: BlobOrTree2::Tree(boxed_tree),
//                         name: tree_entry.name().unwrap_or("").to_string(),
//                     })
//                 }
//                 _ => panic!("Unkown object type."),
//             }
//         })
//         .collect::<Vec<BlobOrTree<'_>>>()
// }

// fn print_tree_commits(repo: &Repository, tree: Vec<BlobOrTree<'_>>) {
//     for e in tree {
//         match e.__inner {
//             BlobOrTree2::Blob(_) => {
//                 println!("* {:?}", get_file_or_tree_latest_commit(&repo, None, e))
//             }
//             BlobOrTree2::Tree(tree) => print_tree_commits(
//                 &repo,
//                 tree.into_iter()
//                     .map(|boxed| *boxed)
//                     .collect::<Vec<BlobOrTree<'_>>>(),
//             ),
//         }
//     }
// }

#[tokio::main]
async fn main() -> Result<()> {
    println!("Getting test repo files and inserting commits.");
    let git_repo = match Repository::open_bare("./tests/ibx34/why-no-work") {
        Ok(repo) => repo,
        Err(err) => {
            panic!("{err:?}");
        }
    };
    println!("Repo HEAD --> {:?}", git_repo.head().unwrap().target());
    let requested_branch = git_repo.find_branch("master", Local).unwrap();

    let _tree = requested_branch.get().peel_to_tree().unwrap();

    // let tree = handle_tree(&git_repo, _tree);
    // print_tree_commits(&git_repo, tree);
    // let files_and_parents = get_file_or_tree_owner(&git_repo, None, tree);

    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::INFO)
    //     .pretty()
    //     .init();

    let app = App::init().await?;
    // throw the result away, we dont care.
    let _ = app.create_account("ibx34", "haha").await;

    let repositories = Router::new()
        .route("/create", post(create_repo))
        .route("/:owner/:name", get(get_repo))
        .route("/:owner/:name/tree/:branch", get(get_repo_tree));
    let api = Router::new()
        .route("/:user/:repo/git-upload-pack", post(upload_pack))
        .route("/:user/:repo/git-receive-pack", post(recieve_pack))
        .route("/:user/:repo/info/refs", get(info_refs))
        .nest("/repos", repositories);
    //.layer(from_fn_with_state(app_state.clone(), session_middleware));
    let web_routes = register_web_routes();
    let app = Router::new()
        .nest_service(
            "/assets",
            get_service(ServeDir::new("/home/alfredo/kidney-stones/assets")),
        )
        .nest_service(
            "/dist",
            get_service(ServeDir::new("/home/alfredo/kidney-stones/web/dist/")),
        )
        // git remote add <name> http://127.0.0.1:8080/ibx34/<repo-name>[.git]
        .nest("/api", api)
        .nest("/", web_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(app);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Starting on: {addr:?}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
    println!("Started");
    Ok(())
}
