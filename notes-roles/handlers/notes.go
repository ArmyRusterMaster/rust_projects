package handlers

import (
	"net/http"
	"strconv"

	"notes-roles/db"

	"github.com/gin-gonic/gin"
)

func ShowNotesPage(c *gin.Context) {
	userID := c.GetInt("user_id")
	username, _ := c.Get("username")
	notes, err := db.GetNotesByUser(userID)
	if err != nil {
		c.HTML(http.StatusInternalServerError, "notes.html", gin.H{
			"title":    "Мои заметки",
			"username": username,
			"error":   "Ошибка загрузки заметок",
		})
		return
	}
	c.HTML(http.StatusOK, "notes.html", gin.H{
		"title":    "Мои заметки",
		"username": username,
		"notes":    notes,
		"owner":    "self",
	})
}

func HandleCreateNote(c *gin.Context) {
	userID := c.GetInt("user_id")
	title := c.PostForm("title")
	content := c.PostForm("content")
	_, err := db.CreateNote(userID, title, content)
	if err != nil {
		username, _ := c.Get("username")
		notes, _ := db.GetNotesByUser(userID)
		c.HTML(http.StatusInternalServerError, "notes.html", gin.H{
			"title":    "Мои заметки",
			"username": username,
			"notes":    notes,
			"error":   "Не удалось создать заметку",
		})
		return
	}
	c.Redirect(http.StatusFound, "/notes")
}

func HandleDeleteNote(c *gin.Context) {
	userID := c.GetInt("user_id")
	noteID, _ := strconv.Atoi(c.Param("id"))
	err := db.DeleteNote(userID, noteID)
	if err != nil {
		c.Redirect(http.StatusFound, "/notes?error=delete_failed")
		return
	}
	c.Redirect(http.StatusFound, "/notes")
}

func HandleUpdateNote(c *gin.Context) {
	userID := c.GetInt("user_id")
	noteID, _ := strconv.Atoi(c.Param("id"))
	title := c.PostForm("title")
	content := c.PostForm("content")
	err := db.UpdateNote(userID, noteID, title, content)
	if err != nil {
		c.Redirect(http.StatusFound, "/notes?error=update_failed")
		return
	}
	c.Redirect(http.StatusFound, "/notes")
}
