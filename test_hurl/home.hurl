POST http://localhost:8000/_api.json/home.set
{
	"title": "Great Title",
	"description": "",
	"body": ""
}
HTTP 400


POST http://localhost:8000/_api.json/home.set
{
	"title": "New great Title",
	"description": "desc",
	"body": "Foo bar."
}
HTTP 200


POST http://localhost:8000/_api.json/home.get
HTTP 200
{"title":"New great Title","description":"desc","body":"Foo bar."}
