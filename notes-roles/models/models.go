package models

import "time"

type User struct {
	ID       int    `json:"id"`
	Username string `json:"username"`
	Password string `json:"-"`
	Role     string `json:"role"` // "user" или "admin"
}

type Note struct {
	ID      int       `json:"id"`
	UserID  int       `json:"user_id"`
	Title   string    `json:"title"`
	Content string  `json:"content"`
	Created time.Time `json:"created"`
}
