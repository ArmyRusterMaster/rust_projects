package main

import (
	"embed"
	"html/template"
	"log"
	"net/http"
	"os"

	"notes-roles/db"
	"notes-roles/handlers"
	"notes-roles/middleware"

	"github.com/gin-gonic/gin"
)

//go:embed templates/*
var templatesFS embed.FS

func main() {
	db.Init()
	defer db.Conn.Close()

	r := gin.Default()
	tmpl := template.Must(template.New("").ParseFS(templatesFS, "templates/*.html"))
	r.SetHTMLTemplate(tmpl)

	// Открытые маршруты
	r.GET("/", handlers.ShowLanding)
	r.GET("/login", handlers.ShowLogin)
	r.POST("/login", handlers.HandleLogin)
	r.GET("/register", handlers.ShowRegister)
	r.POST("/register", handlers.HandleRegister)

	// Защищённые маршруты
	protected := r.Group("/")
	protected.Use(middleware.AuthRequired())
	{
		protected.GET("/dashboard", func(c *gin.Context) {
			c.HTML(http.StatusOK, "dashboard.html", gin.H{
				"title":    "Личный кабинет",
				"username": c.GetString("username"),
				"role":     c.GetString("role"),
			})
		})
		protected.GET("/notes", handlers.ShowNotesPage)
		protected.POST("/notes", handlers.HandleCreateNote)
		protected.POST("/notes/:id/delete", handlers.HandleDeleteNote)
		protected.POST("/notes/:id/update", handlers.HandleUpdateNote)
		protected.GET("/logout", handlers.HandleLogout)
		protected.POST("/delete-account", handlers.HandleDeleteAccount)
	}

	// Административные маршруты
	admin := r.Group("/admin")
	admin.Use(middleware.AuthRequired(), middleware.AdminRequired())
	{
		admin.GET("/users", handlers.AdminListUsers)
		admin.GET("/users/:id", handlers.AdminViewUserNotes)
		admin.POST("/users/:id", handlers.AdminCreateNoteForUser)
		admin.POST("/users/:id/notes/:noteID/delete", handlers.AdminDeleteUserNote)
		admin.POST("/users/:id/notes/:noteID/update", handlers.AdminUpdateUserNote)
	}

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("Сервер запущен на http://localhost:%s", port)
	r.Run(":" + port)
}
