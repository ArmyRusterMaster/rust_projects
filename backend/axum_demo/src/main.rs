use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{CookieJar, Cookie};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
enum UserRole {
    Admin,
    Regular,
}

#[derive(Clone, Serialize, Deserialize)]
struct User {
    id: Uuid,
    username: String,
    password_hash: String,
    role: UserRole,
}

#[derive(Clone, Serialize, Deserialize)]
struct Post {
    id: Uuid,
    title: String,
    content: String,
    author_id: Uuid,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct AppState {
    users: HashMap<Uuid, User>,
    posts: HashMap<Uuid, Post>,
    sessions: HashMap<String, Uuid>,
}

type SharedState = Arc<RwLock<AppState>>;

async fn get_user_id_from_cookie(jar: &CookieJar, state: &AppState) -> Option<Uuid> {
    let session_id = jar.get("session_id")?.value().to_string();
    state.sessions.get(&session_id).copied()
}

async fn get_current_user(jar: &CookieJar, state: &AppState) -> Option<User> {
    let user_id = get_user_id_from_cookie(jar, state).await?;
    state.users.get(&user_id).cloned()
}

fn render_page(title: &str, content: String, user: Option<&User>) -> Html<String> {
    let nav = if let Some(u) = user {
        format!(
            r#"<a href="/dashboard">Dashboard</a> | <a href="/logout">Logout ({})</a>"#,
            u.username
        )
    } else {
        r#"<a href="/">Home</a> | <a href="/login">Login</a> | <a href="/register">Register</a>"#
            .to_string()
    };
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>{title}</title></head>
<body>
    <nav>{nav}</nav><hr>
    {body}
</body>
</html>"#,
        title = title,
        nav = nav,
        body = content
    );
    Html(html)
}

async fn home(jar: CookieJar, State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    if get_current_user(&jar, &state).await.is_some() {
        Redirect::to("/dashboard").into_response()
    } else {
        render_page(
            "Home",
            "<h1>Welcome to Axum Demo</h1><p>Please login or register.</p>".into(),
            None,
        )
        .into_response()
    }
}

#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    password: String,
    role: String,
}

async fn register_form() -> impl IntoResponse {
    let content = r#"
    <h1>Register</h1>
    <form method="post" action="/register">
        <label>Username: <input name="username" required></label><br>
        <label>Password: <input type="password" name="password" required></label><br>
        <label>Role:
            <select name="role">
                <option value="user">Regular User</option>
                <option value="admin">Admin</option>
            </select>
        </label><br>
        <button type="submit">Register</button>
    </form>
    "#;
    render_page("Register", content.into(), None)
}

async fn register(
    State(state): State<SharedState>,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    let mut state = state.write().await;
    if state.users.values().any(|u| u.username == form.username) {
        return render_page(
            "Register Error",
            "<p>Username already taken. <a href='/register'>Try again</a></p>".into(),
            None,
        )
        .into_response();
    }
    let role = if form.role == "admin" {
        UserRole::Admin
    } else {
        UserRole::Regular
    };
    let id = Uuid::new_v4();
    let password_hash = bcrypt::hash(&form.password, bcrypt::DEFAULT_COST).unwrap();
    state.users.insert(
        id,
        User {
            id,
            username: form.username,
            password_hash,
            role,
        },
    );
    Redirect::to("/login").into_response()
}

async fn login_form() -> impl IntoResponse {
    render_page(
        "Login",
        r#"<h1>Login</h1>
        <form method="post" action="/login">
            <label>Username: <input name="username" required></label><br>
            <label>Password: <input type="password" name="password" required></label><br>
            <button type="submit">Login</button>
        </form>"#
            .into(),
        None,
    )
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    jar: CookieJar,
    State(state): State<SharedState>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let user_id = {
        let state = state.read().await;
        state
            .users
            .values()
            .find(|u| u.username == form.username && bcrypt::verify(&form.password, &u.password_hash).unwrap_or(false))
            .map(|u| u.id)
    };
    if let Some(uid) = user_id {
        let mut state = state.write().await;
        let session_id = Uuid::new_v4().to_string();
        state.sessions.insert(session_id.clone(), uid);
        let cookie = Cookie::build(("session_id", session_id))
            .path("/")
            .http_only(true)
            .build();
        (jar.add(cookie), Redirect::to("/dashboard")).into_response()
    } else {
        render_page(
            "Login Error",
            "<p>Invalid credentials. <a href='/login'>Try again</a></p>".into(),
            None,
        )
        .into_response()
    }
}

async fn logout(jar: CookieJar, State(state): State<SharedState>) -> impl IntoResponse {
    let mut state = state.write().await;
    if let Some(sid) = jar.get("session_id").map(|c| c.value().to_string()) {
        state.sessions.remove(&sid);
    }
    (jar.remove(Cookie::from("session_id")), Redirect::to("/"))
}

async fn dashboard(jar: CookieJar, State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    let user = match get_current_user(&jar, &state).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let mut content = format!(
        "<h1>Dashboard - {} ({:?})</h1>",
        user.username, user.role
    );
    content.push_str("<p><a href='/posts/new'>Create new post</a></p><h2>Posts</h2><ul>");
    let posts: Vec<&Post> = if user.role == UserRole::Admin {
        state.posts.values().collect()
    } else {
        state
            .posts
            .values()
            .filter(|p| p.author_id == user.id)
            .collect()
    };
    for post in posts {
        let author = state
            .users
            .get(&post.author_id)
            .map(|u| u.username.as_str())
            .unwrap_or("unknown");
        content.push_str(&format!(
            "<li><strong>{title}</strong> by {author} \
             <a href='/posts/edit/{pid}'>Edit</a> \
             <form method='post' action='/posts/delete/{pid}' style='display:inline;'>
               <button type='submit'>Delete</button>
             </form></li>",
            title = post.title,
            author = author,
            pid = post.id,
        ));
    }
    content.push_str("</ul>");
    render_page("Dashboard", content, Some(&user)).into_response()
}

async fn new_post_form(jar: CookieJar, State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    let user = match get_current_user(&jar, &state).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    render_page(
        "New Post",
        r#"<h1>New Post</h1>
        <form method="post" action="/posts/create">
            <label>Title: <input name="title" required></label><br>
            <label>Content:<br><textarea name="content" required></textarea></label><br>
            <button type="submit">Create</button>
        </form>"#
            .into(),
        Some(&user),
    )
    .into_response()
}

#[derive(Deserialize)]
struct PostForm {
    title: String,
    content: String,
}

async fn create_post(
    jar: CookieJar,
    State(state): State<SharedState>,
    Form(form): Form<PostForm>,
) -> impl IntoResponse {
    let user_id = {
        let state = state.read().await;
        match get_user_id_from_cookie(&jar, &state).await {
            Some(id) => id,
            None => return Redirect::to("/login").into_response(),
        }
    };
    let mut state = state.write().await;
    let post = Post {
        id: Uuid::new_v4(),
        title: form.title,
        content: form.content,
        author_id: user_id,
    };
    state.posts.insert(post.id, post);
    Redirect::to("/dashboard").into_response()
}

async fn edit_post_form(
    jar: CookieJar,
    Path(post_id): Path<Uuid>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let state = state.read().await;
    let user = match get_current_user(&jar, &state).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    if let Some(post) = state.posts.get(&post_id) {
        if user.role != UserRole::Admin && post.author_id != user.id {
            return (StatusCode::FORBIDDEN, "Access denied").into_response();
        }
        let content = format!(
            r#"<h1>Edit Post</h1>
            <form method="post" action="/posts/update/{}">
                <label>Title: <input name="title" value="{}" required></label><br>
                <label>Content:<br><textarea name="content" required>{}</textarea></label><br>
                <button type="submit">Update</button>
            </form>"#,
            post.id, post.title, post.content
        );
        return render_page("Edit Post", content, Some(&user)).into_response();
    }
    (StatusCode::NOT_FOUND, "Post not found").into_response()
}

#[derive(Deserialize)]
struct UpdatePostForm {
    title: String,
    content: String,
}

async fn update_post(
    jar: CookieJar,
    Path(post_id): Path<Uuid>,
    State(state): State<SharedState>,
    Form(form): Form<UpdatePostForm>,
) -> impl IntoResponse {
    {
        let state = state.read().await;
        let user_id = match get_user_id_from_cookie(&jar, &state).await {
            Some(id) => id,
            None => return Redirect::to("/login").into_response(),
        };
        if let Some(post) = state.posts.get(&post_id) {
            let user = state.users.get(&user_id).unwrap();
            if user.role != UserRole::Admin && post.author_id != user_id {
                return (StatusCode::FORBIDDEN, "Access denied").into_response();
            }
        } else {
            return (StatusCode::NOT_FOUND, "Post not found").into_response();
        }
    }
    let mut state = state.write().await;
    if let Some(post) = state.posts.get_mut(&post_id) {
        post.title = form.title;
        post.content = form.content;
    }
    Redirect::to("/dashboard").into_response()
}

async fn delete_post(
    jar: CookieJar,
    Path(post_id): Path<Uuid>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    {
        let state = state.read().await;
        let user_id = match get_user_id_from_cookie(&jar, &state).await {
            Some(id) => id,
            None => return Redirect::to("/login").into_response(),
        };
        if let Some(post) = state.posts.get(&post_id) {
            let user = state.users.get(&user_id).unwrap();
            if user.role != UserRole::Admin && post.author_id != user_id {
                return (StatusCode::FORBIDDEN, "Access denied").into_response();
            }
        } else {
            return (StatusCode::NOT_FOUND, "Post not found").into_response();
        }
    }
    let mut state = state.write().await;
    state.posts.remove(&post_id);
    Redirect::to("/dashboard").into_response()
}

#[tokio::main]
async fn main() {
    // ⬇️ ЗАГРУЗКА СОХРАНЁННЫХ ДАННЫХ
    let initial_state: AppState = match tokio::fs::read_to_string("data.json").await {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => AppState::default(),
    };
    let shared_state: SharedState = Arc::new(RwLock::new(initial_state));

    // ⬇️ СОХРАНЕНИЕ ПРИ ЗАВЕРШЕНИИ (Ctrl+C)
    let state_for_save = shared_state.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        let state = state_for_save.read().await;
        if let Ok(json) = serde_json::to_string_pretty(&*state) {
            let _ = tokio::fs::write("data.json", json).await;
        }
        std::process::exit(0);
    });

    let app = Router::new()
        .route("/", get(home))
        .route("/register", get(register_form).post(register))
        .route("/login", get(login_form).post(login))
        .route("/logout", get(logout))
        .route("/dashboard", get(dashboard))
        .route("/posts/new", get(new_post_form))
        .route("/posts/create", post(create_post))
        .route("/posts/edit/{id}", get(edit_post_form))
        .route("/posts/update/{id}", post(update_post))
        .route("/posts/delete/{id}", post(delete_post))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
