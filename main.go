package main

import (
	"os"

	"github.com/kerinin/hammer/web"
	"github.com/kerinin/hammer/db"
	"github.com/codegangsta/cli"
)

func main() {
	app := cli.NewApp()

	app.Name = "hammer"
	app.Usage = "Hamming search DB"

	app.Commands = []cli.Command{
		{
			Name:      "serve",
			ShortName: "s",
			Usage:     "Start an HTTP server",
			Flags: []cli.Flag {
				cli.StringFlag{
					Name: "bind",
					Value: "localhost:3000",
					Usage: "HTTP bind host and port",
					// EnvVar: "HAMMER_BIND",
				},
				cli.IntFlag{
					Name: "bits, b",
					Value: 64,
					Usage: "key size",
					// EnvVar: "HAMMER_BITS",
				},
				cli.IntFlag{
					Name: "tolerance, t",
					Value: 3,
					Usage: "return keys with hamming distance equal-to or less-than this",
					// EnvVar: "HAMMER_TOLERANCE",
				},
				cli.IntFlag{
					Name: "max-keys, m",
					Value: 100000,
					Usage: "the maximum number of keys to keep in memory",
					// EnvVar: "HAMMER_MAX_KEYS",
				},
			},
			Action: func(c *cli.Context) {
				bind := c.String("bind")
				bits := uint(c.Int("bits"))
				tolerance := uint(c.Int("tolerance"))
				max_keys := c.Int("max-keys")

				database := db.NewPartitioning(bits, tolerance, max_keys)

				web.NewServer(bind, database).Run()
			},
		},
	}

	app.Run(os.Args)
}
