package db

import (
	"database/sql"
	"log"

	"notes-roles/models"

	_ "modernc.org/sqlite"
)

var Conn *sql.DB

func Init() {
	var err error
	Conn, err = sql.Open("sqlite", "file:notes.db?cache=shared&_journal_mode=WAL")
	if err != nil {
		log.Fatal(err)
	}
	_, err = Conn.Exec(`CREATE TABLE IF NOT EXISTS users (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		username TEXT UNIQUE NOT NULL,
		password TEXT NOT NULL,
		role TEXT NOT NULL DEFAULT 'user'
	)`)
	if err != nil {
		log.Fatal(err)
	}
	_, err = Conn.Exec(`CREATE TABLE IF NOT EXISTS notes (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		user_id INTEGER NOT NULL,
		title TEXT NOT NULL,
		content TEXT NOT NULL,
		created DATETIME DEFAULT CURRENT_TIMESTAMP,
		FOREIGN KEY (user_id) REFERENCES users(id)
	)`)
	if err != nil {
		log.Fatal(err)
	}
}

func GetUserByUsername(username string) (*models.User, error) {
	u := &models.User{}
	err := Conn.QueryRow("SELECT id, username, password, role FROM users WHERE username = ?", username).
		Scan(&u.ID, &u.Username, &u.Password, &u.Role)
	if err != nil {
		return nil, err
	}
	return u, nil
}

func CreateUser(username, passwordHash, role string) (int64, error) {
	res, err := Conn.Exec("INSERT INTO users(username, password, role) VALUES (?, ?, ?)", username, passwordHash, role)
	if err != nil {
		return 0, err
	}
	return res.LastInsertId()
}

func DeleteUser(userID int) error {
	tx, err := Conn.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()
	_, err = tx.Exec("DELETE FROM notes WHERE user_id = ?", userID)
	if err != nil {
		return err
	}
	_, err = tx.Exec("DELETE FROM users WHERE id = ?", userID)
	if err != nil {
		return err
	}
	return tx.Commit()
}

func GetNonAdminUsers() ([]models.User, error) {
	rows, err := Conn.Query("SELECT id, username, role FROM users WHERE role != 'admin' ORDER BY id")
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var users []models.User
	for rows.Next() {
		var u models.User
		err := rows.Scan(&u.ID, &u.Username, &u.Role)
		if err != nil {
			return nil, err
		}
		users = append(users, u)
	}
	return users, nil
}

// CRUD заметок
func CreateNote(userID int, title, content string) (int64, error) {
	res, err := Conn.Exec("INSERT INTO notes(user_id, title, content) VALUES (?, ?, ?)", userID, title, content)
	if err != nil {
		return 0, err
	}
	return res.LastInsertId()
}

func GetNotesByUser(userID int) ([]models.Note, error) {
	rows, err := Conn.Query("SELECT id, user_id, title, content, created FROM notes WHERE user_id = ? ORDER BY created DESC", userID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var notes []models.Note
	for rows.Next() {
		var n models.Note
		err := rows.Scan(&n.ID, &n.UserID, &n.Title, &n.Content, &n.Created)
		if err != nil {
			return nil, err
		}
		notes = append(notes, n)
	}
	return notes, nil
}

func DeleteNote(userID, noteID int) error {
	_, err := Conn.Exec("DELETE FROM notes WHERE id = ? AND user_id = ?", noteID, userID)
	return err
}

func UpdateNote(userID, noteID int, title, content string) error {
	_, err := Conn.Exec("UPDATE notes SET title = ?, content = ? WHERE id = ? AND user_id = ?", title, content, noteID, userID)
	return err
}
func GetUserByID(userID int) (*models.User, error) {
	u := &models.User{}
	err := Conn.QueryRow("SELECT id, username, password, role FROM users WHERE id = ?", userID).Scan(&u.ID, &u.Username, &u.Password, &u.Role)
	if err != nil {
		return nil, err
	}
	return u, nil
}
