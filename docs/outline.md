# This is an evolving outline for the guide.

**Introduction**: Overarching info about Rocket and the guide.

  * **Foreword** - A bit about the guide.
  * **Audience** - Who this guide is intended for.
  * **Philosophy** - The ideas behind Rocket.
  * **Outline** - An outline of the guide.

**Quickstart**: Super quick info about running the examples.

**Getting Started**: Overview of how to use Rocket with Rust.

  * **Rust Nightly** - How to get Rust nightly, and why Rocket needs it.
  * **Cargo Project** - How to use Rocket in a Cargo project.
    * **Project Organization** - Overview of the three crates: lib, codegen, contrib.
  * **Running** - Running a Rocket application and what to expect.
  * **Configuration** - How to configure Rocket.
    * **Rocket.toml** - Syntax and semantics of the Rocket.toml file.
    * **Environments** - Explanation and how to set.
    * **Logging** - How logging in Rocket works.

**From Request to Response**: Overview of Rocket's request/response handling.

  * **Routing** - How routing works in Rocket.
  * **Request** - How request (pre)processing works.
  * **Response** - How response (post) processing works.

**Routing**: How to route in Rocket.

  * **Overview** - An overview of routes in Rocket.
  * **Route Attribute** - Description, explanation, and how to use the attribute.
    * **Path** - Explanation of the path argument.
      * **Static Parameters** - Static segments in route path.
      * **Dynamic Parameters** - Dynamic segments in route path.
      * **Segment Parameters** - Segment.. segments in route path.
      * **Query Params** - Segment.. segments in route path.
    * **Format** - Explanation of the format attribute.
    * **Form** - Exaplanation of the form attribute.
  * **Handlers** - How to declare handlers in Rocket.
    * **Path Arguments** - Parameters declared in the route path.
    * **FromRequest Arguments** - Parameters derived from the request.
    * **Form Arguments** - Parameters derived from a form.
  * **Mounting** - Mounting routes at a given path.
  * **Collision and Ranking** - Explains route collisions and ranking.
  * **Route Outcome** - Different types of outcomes and their meaning.
  * **Catching Errors** - Explanation of catchers and their use.
  * **Manual Routing** - How to route without using code generation.

**Request**

  * **Overview** - An overview of the section.
  * **Request Traits** - Explanation of traits used by request handlers.
    * **FromParam** - Explanation of the FromParam trait.
    * **FromSegments** - Explanation of the FromSegments trait.
    * **FromRequest** - Explanation of the FromRequest trait.
    * **FromForm** - Explanation of the FromForm trait.
      * **FromFormValue** - Explanation of the FromFormValue trait.
  * **Forms** - How Rocket makes forms easy.
    * **Custom Validation** - How to add custom validation to forms.
    * **Query Params** - How to use/validate query params.
  * **Cookies** - Using cookies.
  * **Session** - Using sessions with a private key.
  * **JSON** - Taking in JSON.

**Response**

  * **Overview** - An overview of the section.
  * **Responder Trait** - Explanation of the Responder trait and its instability.
  * **Simple Responses** - The simple response types Rocket provides.
    * Empty, Redirect, StatusResponse, Data
  * **JSON** - Explanation and use of the JSON response.
  * **Templating** - Templates in contrib.
  * **Flash** - Explanation and use of the Flash response.
  * **Stream** - Explanation and use of the Stream response.
