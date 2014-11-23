package web

import (
	"log"

	"math/big"
	"net/http"
	"encoding/json"

	"github.com/go-martini/martini"
	"github.com/martini-contrib/binding"
	"github.com/kerinin/hammer/db"
)

type Server struct {
	bind string
	database db.Partitioning
}

func NewServer(bind string, database db.Partitioning) *Server {
	return &Server{bind: bind, database: database}
}


func (s *Server) Run() {
	m := martini.Classic()

	m.Post("/add$", binding.Json(AddRequest{}), func(request AddRequest, params martini.Params, logger *log.Logger) (int, string) {
		return s.handleAdd(request, params, logger)
	})

	m.Post("/query$", binding.Json(QueryRequest{}), func(request QueryRequest, params martini.Params, logger *log.Logger) (int, string) {
		return s.handleQuery(request, params, logger)
	})

	m.Post("/delete$", binding.Json(DeleteRequest{}), func(request DeleteRequest, params martini.Params, logger *log.Logger) (int, string) {
		return s.handleDelete(request, params, logger)
	})

	log.Printf("Starting HTTP server for Hammer DB %v on %v", s.database, s.bind)
	log.Fatal(http.ListenAndServe(s.bind, m))
}

func (s *Server) handleAdd(request AddRequest, params martini.Params, logger *log.Logger) (int, string) {
	// logger.Printf("Handling Add with db:%v, request:%v, params:%v", s.database, request, params)

	scalars := request.Scalars
	response := AddResponse{}
	response.Scalars = make([]ScalarAddResult, 0, len(scalars))

	for _, scalar := range request.Scalars {
		safe_scalar := big.NewInt(0)
		safe_scalar.SetBytes(scalar.Bytes())

		added, err := s.database.Insert(safe_scalar)
		if err != nil {
			return 500, err.Error()
		}

		// logger.Printf("Added %016b: %v", safe_scalar, added)

		response.Scalars = append(response.Scalars, ScalarAddResult{Scalar: *safe_scalar, Added: added})
	}

	json_bytes, err := json.Marshal(response)
	if err != nil {
		return 500, err.Error()
	}

	return 200, string(json_bytes)
}

func (s *Server) handleDelete(request DeleteRequest, params martini.Params, logger *log.Logger) (int, string) {
	// logger.Printf("Handling Delete with db:%v, request:%v, params:%v", s.database, request, params)

	scalars := request.Scalars
	response := DeleteResponse{}
	response.Scalars = make([]ScalarDeleteResult, 0, len(scalars))

	for _, scalar := range request.Scalars {
		safe_scalar := big.NewInt(0)
		safe_scalar.SetBytes(scalar.Bytes())

		deleted, err := s.database.Remove(safe_scalar)
		if err != nil {
			return 500, err.Error()
		}

		// logger.Printf("Deleted %016b: %v", safe_scalar, deleted)

		response.Scalars = append(response.Scalars, ScalarDeleteResult{Scalar: *safe_scalar, Deleted: deleted})
	}

	json_bytes, err := json.Marshal(response)
	if err != nil {
		return 500, err.Error()
	}

	return 200, string(json_bytes)
}

func (s *Server) handleQuery(request QueryRequest, params martini.Params, logger *log.Logger) (int, string) {
	// logger.Printf("Handling Query with db:%v, request:%v, params:%v", s.database, request, params)

	scalars := request.Scalars
	response := QueryResponse{}
	response.Scalars = make([]ScalarQueryResult, 0, len(scalars))

	for _, scalar := range request.Scalars {
		safe_scalar := big.NewInt(0)
		safe_scalar.SetBytes(scalar.Bytes())

		found_map, err := s.database.Find(safe_scalar)
		if err != nil {
			return 500, err.Error()
		}

		found := make([]big.Int, 0, len(found_map))
		for i, _ := range found_map {
			found = append(found, *i)
		}

		// logger.Printf("Queried %016b: %v", safe_scalar, found_map)
		response.Scalars = append(response.Scalars, ScalarQueryResult{Scalar: *safe_scalar, Found: found})
	}

	json_bytes, err := json.Marshal(response)
	if err != nil {
		return 500, err.Error()
	}

	return 200, string(json_bytes)
}
