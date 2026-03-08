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
HTTP 403


GET http://localhost:8000/!user-token-editor
HTTP 200
[Captures]
token: body


POST http://localhost:8000/_api.json/home.set
Cookie: user={{token}}
{
	"title": "New great Title",
	"description": "desc",
	"body": "Foo bar."
}
HTTP 200


POST http://localhost:8000/_api.json/home.get
HTTP 200
{"title":"New great Title","description":"desc","body":"Foo bar."}
