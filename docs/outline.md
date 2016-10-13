# This is an evolving outline for the guide.

**Introduction**: Overarching info about Rocket and the guide.

  * **Foreword** - A bit about the guide.
  * **Audience** - Who this guide is intended for.
  * **Philosophy** - The ideas behind Rocket.
  * **Outline** - An outline of the guide.

**Quickstart**: Super quick info cloning the repo and running the examples.

**Getting Started**: Overview of how to use Rocket with Rust.

  * **Rust Nightly** - How to get Rust nightly and why Rocket needs it.
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
  * **Route Attributes** - Description, explanation, and how to use the attribute.
    * **Path** - Explanation of the path argument.
      * **Static Parameters** - Static segments in route path.
      * **Dynamic Parameters** - Dynamic segments in route path.
      * **Segment Parameters** - Segment.. segments in route path.
      * **Query Params** - Query parameter in route path.
    * **Format** - Explanation of the format attribute.
    * **Data** - Exaplanation of the data attribute.
    * **FromRequest Arguments** - Parameters derived from the request.
  * **Mounting** - Mounting routes at a given path.
  * **Collisions and Ranking** - Explains route collisions and ranking.
  * **Responses** - Different types of outcomes and their meaning.
    * **Complete** - What it means for a request to complete.
    * **Forward** - What it means for a request to forward.
  * **Catching Errors** - Explanation of catchers and their use.
  * **Manual Routing** - How to route without using code generation.

**Request**

  * **Overview** - An overview of the section.
  * **Request Traits** - Explanation of traits used by request handlers.
    * **FromParam** - Explanation of the FromParam trait.
    * **FromSegments** - Explanation of the FromSegments trait.
    * **FromRequest** - Explanation of the FromRequest trait.
    * **FromData** - Explanation of the FromParam trait.
    * **FromForm** - Explanation of the FromForm trait.
      * **FromFormValue** - Explanation of the FromFormValue trait.
  * **Data** - How Rocket makes handling streaming data easy.
    * **JSON** - Taking in JSON.
    * **Forms** - Handling url-encoded forms. Details in next section.
  * **Forms** - How Rocket makes forms easy.
    * **Custom Validation** - How to add custom validation to forms.
    * **Query Params** - How to use/validate query params.
  * **Cookies** - Using cookies.
    * **Flash** - Using flash cookies.
  * **Session** - Using sessions with a private key.

**Response**

  * **Overview** - An overview of the section.
  * **Responder Trait** - Explanation of the Responder trait and its instability.
    * **Outcomes** - Different types of responder outcomes.
      * **Success** - What it means for a responder to succeed.
      * **Failure** - What it means for a responder to fail.k
      * **Forward** - What it means for a responder to forward.
  * **Simple Responses** - The simple response types Rocket provides.
    * Redirect, StatusResponse, Failure, Content
  * **NamedFile** - Explanation and use of the NamedFile responder.
  * **Stream** - Explanation and use of the Stream response.
  * **Flash** - Explanation and use of the Flash response.
  * **JSON** - Explanation and use of the JSON responder.
  * **Templating** - Templates in contrib.
