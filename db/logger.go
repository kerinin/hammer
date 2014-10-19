package db

import (
	"github.com/op/go-logging"
)

var logger = logging.MustGetLogger("hammer.db")
var logFormat = logging.MustStringFormatter("[db] %{level} %{color}%{message}%{color:reset}")

func init() {
	logging.SetFormatter(logFormat)
	logging.SetLevel(logging.WARNING, "hammer.db")
}
