package handlers

import (
	"net/http"
	"strconv"

	"notes-roles/db"

	"github.com/gin-gonic/gin"
)

func AdminListUsers(c *gin.Context) {
	users, err := db.GetNonAdminUsers()
	if err != nil {
		c.HTML(http.StatusInternalServerError, "admin_users.html", gin.H{"title": "Управление пользователями", "error": "Ошибка загрузки списка"})
		return
	}
	c.HTML(http.StatusOK, "admin_users.html", gin.H{
		"title":    "Пользователи",
		"users":    users,
		"username": c.GetString("username"),
	})
}

func AdminViewUserNotes(c *gin.Context) {
	targetUserID, err := strconv.Atoi(c.Param("id"))
	if err != nil {
		c.Redirect(http.StatusFound, "/admin/users")
		return
	}
	targetUser, err := db.GetUserByID(targetUserID)
	if err != nil {
		c.Redirect(http.StatusFound, "/admin/users")
		return
	}
	notes, _ := db.GetNotesByUser(targetUserID)
	c.HTML(http.StatusOK, "notes.html", gin.H{
		"title":    "Заметки пользователя " + targetUser.Username,
		"username": c.GetString("username"),
		"notes":    notes,
		"owner":    "admin",
		"targetID": targetUserID,
	})
}

func AdminCreateNoteForUser(c *gin.Context) {
	targetUserID, _ := strconv.Atoi(c.Param("id"))
	title := c.PostForm("title")
	content := c.PostForm("content")
	db.CreateNote(targetUserID, title, content)
	c.Redirect(http.StatusFound, "/admin/users/"+strconv.Itoa(targetUserID))
}

func AdminDeleteUserNote(c *gin.Context) {
	targetUserID, _ := strconv.Atoi(c.Param("id"))
	noteID, _ := strconv.Atoi(c.Param("noteID"))
	db.DeleteNote(targetUserID, noteID)
	c.Redirect(http.StatusFound, "/admin/users/"+strconv.Itoa(targetUserID))
}

func AdminUpdateUserNote(c *gin.Context) {
	targetUserID, _ := strconv.Atoi(c.Param("id"))
	noteID, _ := strconv.Atoi(c.Param("noteID"))
	title := c.PostForm("title")
	content := c.PostForm("content")
	db.UpdateNote(targetUserID, noteID, title, content)
	c.Redirect(http.StatusFound, "/admin/users/"+strconv.Itoa(targetUserID))
}
