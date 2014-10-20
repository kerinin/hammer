package web

import (
	"sync"

	"github.com/go-martini/martini"
	"github.com/martini-contrib/binding"
	"github.com/kerinin/hammer/db"
)

var databases map[string]db.Partitioning
var databases_mutex sync.RWMutex

func Server() {
	databases = make(map[string]db.Partitioning)
	databases_mutex = sync.RWMutex{}

	m := martini.Classic()

	m.Post("/db/(?P<bits>\\d+)/(?P<tolerance>\\d+)/(?P<namespace>.*)/bulk_add$", binding.Json(AddRequest{}), addHandler)
	m.Post("/db/(?P<bits>\\d+)/(?P<tolerance>\\d+)/(?P<namespace>.*)/bulk_query$", binding.Json(QueryRequest{}), queryHandler)
	m.Post("/db/(?P<bits>\\d+)/(?P<tolerance>\\d+)/(?P<namespace>.*)/bulk_delete$", binding.Json(DeleteRequest{}), deleteHandler)

	m.Run()
}
