package handlers

import (
	"net/http"
	"time"

	"notes-roles/db"
	"notes-roles/middleware"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"
)

func ShowLanding(c *gin.Context) {
	_, err := c.Cookie("token")
	if err == nil {
		c.Redirect(http.StatusFound, "/dashboard")
		return
	}
	c.Redirect(http.StatusFound, "/login")
}

func ShowLogin(c *gin.Context) {
	c.HTML(http.StatusOK, "login.html", gin.H{"title": "Вход"})
}

func HandleLogin(c *gin.Context) {
	username := c.PostForm("username")
	password := c.PostForm("password")
	user, err := db.GetUserByUsername(username)
	if err != nil || bcrypt.CompareHashAndPassword([]byte(user.Password), []byte(password)) != nil {
		c.HTML(http.StatusUnauthorized, "login.html", gin.H{"title": "Вход", "error": "Неверное имя пользователя или пароль"})
		return
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"user_id":  user.ID,
		"username": user.Username,
		"role":     user.Role,
		"exp":      time.Now().Add(24 * time.Hour).Unix(),
	})
	tokenString, _ := token.SignedString(middleware.JwtSecret)
	middleware.SetAuthCookie(c, tokenString)
	c.Redirect(http.StatusFound, "/dashboard")
}

func ShowRegister(c *gin.Context) {
	c.HTML(http.StatusOK, "register.html", gin.H{"title": "Регистрация"})
}

func HandleRegister(c *gin.Context) {
	username := c.PostForm("username")
	password := c.PostForm("password")
	role := c.PostForm("role")
	if role == "" {
		role = "user"
	}
	if role != "user" && role != "admin" {
		role = "user"
	}
	if len(password) < 6 {
		c.HTML(http.StatusBadRequest, "register.html", gin.H{"title": "Регистрация", "error": "Пароль должен содержать минимум 6 символов"})
		return
	}
	hashed, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		c.HTML(http.StatusInternalServerError, "register.html", gin.H{"title": "Регистрация", "error": "Ошибка сервера"})
		return
	}
	id, err := db.CreateUser(username, string(hashed), role)
	if err != nil {
		c.HTML(http.StatusConflict, "register.html", gin.H{"title": "Регистрация", "error": "Имя пользователя занято"})
		return
	}
	// Автоматический вход после регистрации
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"user_id":  id,
		"username": username,
		"role":     role,
		"exp":      time.Now().Add(24 * time.Hour).Unix(),
	})
	tokenString, _ := token.SignedString(middleware.JwtSecret)
	middleware.SetAuthCookie(c, tokenString)
	c.Redirect(http.StatusFound, "/dashboard")
}

func HandleLogout(c *gin.Context) {
	middleware.ClearAuthCookie(c)
	c.Redirect(http.StatusFound, "/login?logged_out=1")
}

func HandleDeleteAccount(c *gin.Context) {
	userID := c.GetInt("user_id")
	err := db.DeleteUser(userID)
	if err != nil {
		c.HTML(http.StatusInternalServerError, "dashboard.html", gin.H{"title": "Ошибка", "error": "Не удалось удалить аккаунт"})
		return
	}
	middleware.ClearAuthCookie(c)
	c.Redirect(http.StatusFound, "/register?deleted=1")
}
