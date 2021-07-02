# Backend

## Peer-Review-Platform

### 📦 Requirements 

* Rust (Nightly)

* MySQL (Tested with version 8.0.23 under WSL2)

  Before running the backend make sure that the Event Scheduler is turned on!

### :rocket: Run

For convenience checkout the latest releases.

If you want to run it manually use 

```
cargo run
```

### 🧰 Configuration

The backend can be configured through the `Rocket.toml` file.

```toml
[global]
# Clear all data (but not events)
db_clear = true
# Insert some mock data
db_insert_mock_data = true
# Set timespan for reviews
review_time_days = 0
review_time_hours = 0
review_time_minutes = 5
# Rocket internal db config
# Please specify a correct db string!
[global.databases]
iprp_db = { url = "mysql://root@127.0.0.1:3306/iprp" }
```

The first two properties allow one to clear the database on start-up and/or insert some mock data.

> Note: A user `admin` with password `admin` is always created when not already exists

The review time properties specify the duration for reviews. The aggregated time is then used by the system.

The last property asks for a connection string to your MySQL database. 

> Note: The database must be created manually before starting the Backend.

### 🗺️ API

Type Definitions:

* `<s>` - String
  * `<sr>` - "teacher" | "student"
  * `<sp>` - "point" | "grade" | "percentage" | "truefalse"
* `<b>` - Boolean
* `<i>` - Integer
* `<f>` - Float
* `<d>` - Date      
  Expected format is ISO 8601 without time zone like `2007-04-05T14:30:30`     
  In JavaScript it can be achieved like this: `(new Date()).toISOString().split(".")[0]`

#### Authentication

<table>
<thead>
<tr>
<th>Route</th>
<th>Method</th>
<th>Input</th>
<th>Output</th>
<th>Notes</th>
</tr>
<tr>
  <td><code>/login</code></td>
  <td>POST</td>
  <td>Basic Auth Header</td>
  <td>
<pre lang=json>
{ "id": &lt;i>, "role": &lt;sr> }
</pre>
  </td>
	<td>Also returns a Session Cookie!</td>
</tr>
  <td><code>/logout</code></td>
  <td>POST</td>
  <td>Session Cookie</td>
  <td>Status Code</td>
  <td></td>
</tr>
</thead>
</table>

#### Teacher

<table>
<thead>
<tr>
<th>Route</th>
<th>Method</th>
<th>Input</th>
<th>Output</th>
<th>Notes</th>
</tr>
</thead>
<tbody>
<tr>
  <td>Show all workshops<br><code>/teacher/workshops</code></td>
  <td>GET</td>
  <td></td>
  <td>
<pre lang=json>
{ 
  "workshops": [ 
    { "id": &lt;i>, "title": &lt;s>, } , ... 
  ]
}
</pre>
  </td>
	<td></td>
</tr>
<tr>
  <td>Retrieve specific workshop<br><code>/teacher/workshop/{id}</code></td>
  <td>GET</td>
  <td></td>
  <td>
<pre lang=json>
{ 
  "ok": &lt;b>,
  "workshop": {
    "title": &lt;s>, "content": &lt;s>, 
    "end": &lt;d>, "anonymous": &lt;b>
    "teachers": [ 
      { 
        "id": &lt;i>, "firstname": &lt;s>, 
        "lastname": &lt;s> 
      }, .. 
    ],
    "students": [ 
      { 
        "id": &lt;i>, "firstname": &lt;s>, 
        "lastname": &lt;s>, "group": &lt;s>,
        "submissions": [ 
          {
            "id": &lt;i>, "date": &lt;d>, 
            "title": &lt;s>, "reviewsDone": &lt;b>,
            "studentid": &lt;i>, "noReviews": &lt;b>,
            "points": &lt;f>, "maxPoints": &lt;f>
          }, ..
        ] 
      } 
    ],
    "criteria": [
      {
        "id": &lt;i>, "type": &lt;sp>,
        "title": &lt;s>, "content": &lt;s>,
        "weight": &lt;f>
      }, ..
    ]
  }
}
</pre>
  </td>
  <td>Points & maxPoints only accessible when reviewsDone true and noReviews false</td>
</tr>
<tr>
  <td>Create new workshop<br><code>/teacher/workshop</code></td>
  <td>POST</td>
  <td>
<pre lang=json>
{ 
  "title": &lt;s>, "content": &lt;s>,
  "end": &lt;d>, "anonymous": &lt;b>,
  "teachers": [ &lt;i>, ..],
  "students": [ &lt;i>, ..],
  "criteria": [ {
    "type": &lt;sp>, "title": &lt;s>,
    "content": &lt;s>, "weight": &lt;f>}, .. ] 
}
</pre>
  </td>
  <td>
<pre lang=json>
{ 
  "ok": &lt;b>,
  "id": &lt;i>
}
</pre>
  </td>
  <td>Teachers & Students array consists of User Ids</td>
</tr>
<tr>
  <td>Update existing workshop<br><code>/teacher/workshop/{id}</code></td>
  <td>PUT</td>
  <td>
<pre lang=json>
{ 
  "title": &lt;s>, "content": &lt;s>,
  "end": &lt;d>, 
  "teachers": [ &lt;i>, ..],
  "students": [ &lt;i>, ..],
  "criteria": [ {
    "type": &lt;sp>, "title": &lt;s>,
    "content": &lt;s>, "weight": &lt;f>}, .. ] 
}
</pre>
  </td>
  <td>
<pre lang=json>
{ 
  "ok": &lt;b>
}
</pre>
  </td>
  <td></td>
</tr>
<tr>
  <td>Delete existing workshop<br><code>/teacher/workshop/{id}</code></td>
  <td>DELETE</td>
  <td></td>
  <td>
<pre lang=json>
{ 
  "ok": &lt;b>
}
</pre>
  </td>
  <td></td>
</tr>
<tr>
  <td>Get Student from Id<br><code>/teacher/search/student</code></td>
  <td>GET</td>
  <td>
Query Parameter: <code>?id=&lt;i></code>
  </td>
  <td>
<pre lang=json>
{ 
  "ok": &lt;b>,
  "id": &lt;i>, "firstname": &lt;s>, "lastname": &lt;s>
}
</pre>
  </td>
  <td></td>
</tr>
<tr>
  <td>Get Student Id from Name<br><code>/teacher/search/student</code></td>
  <td>GET</td>
  <td>
  Query Parameter: <code>?firstname=&lt;s>&lastname=&lt;s></code>
  </td>
  <td>
<pre lang=json>
{ 
  "ok": &lt;b>,
  "id": &lt;i>
}
</pre>
  </td>
  <td></td>
</tr>
<tr>
  <td>Get all Students from a Group<br><code>/teacher/search/student</code></td>
  <td>GET</td>
  <td>
Query Parameter: <code>?group=&lt;s></code>
  </td>
  <td>
<pre lang=json>
{ 
  "ok": &lt;b>,
  "ids": [&lt;i>, ..]
}
</pre>
  </td>
  <td></td>
</tr>
<tr>
  <td>Get all Students<br><code>/teacher/search/student</code></td>
  <td>GET</td>
  <td>
Query Parameter: <code>?all=&lt;b></code>
  </td>
  <td>
<pre lang=json>
{ 
  "ok": &lt;b>,
  "students": [
    {
      "firstname": &lt;s>, "lastname": &lt;s>,
      "id": &lt;i>, "group": &lt;s>
    }, ..
  ]
}
</pre>
  </td>
  <td>All needs to be "true" (<code>?all=true</code>)</td>
</tr>
</tbody>
</table>

#### Student

<table>
<thead>
<tr>
<th>Route</th>
<th>Method</th>
<th>Input</th>
<th>Output</th>
<th>Notes</th>
</tr>
</thead>
<tbody>
<tr>
  <td>Show all workshops<br><code>/student/workshops</code></td>
  <td>GET</td>
  <td></td>
  <td>
<pre lang=json>
{ 
  "workshops": [ 
    { "id": &lt;i>, "title": &lt;s>, } , ... 
  ]
}
</pre>
  </td>
	<td></td>
</tr>
<tr>
  <td>Retrieve specific workshop<br><code>/student/workshop/{id}</code></td>
  <td>GET</td>
  <td></td>
  <td>
<pre lang=json>
{ 
  "ok": &lt;b>,
  "workshop": {
    "title": &lt;s>, "content": &lt;s>, 
    "end": &lt;d>,
    "teachers": [ 
      { 
        "id": &lt;i>, "firstname": &lt;s>, 
        "lastname": &lt;s> 
      }, .. 
    ],
    "students": [ 
      { 
        "id": &lt;i>, "firstname": &lt;s>, 
        "lastname": &lt;s>, "group": &lt;s>
      } 
    ],
    "submissions": [ 
      {
        "id": &lt;i>, "title": &lt;s>, 
        "date": &lt;d>, "locked": &lt;b>,
        "reviewsDone": &lt;b>, "noReviews": &lt;b>
        "points": &lt;f>, "maxPoints": &lt;f>
      }, ..
    ],
    "reviews": [
      {
        "id": &lt;i>, "done": &lt;b>,
        "deadline": &lt;d>, "title": &lt;s>,
        "firstname": &lt;s>, "lastname": &lt;s>
      }
    ]
  }
}
</pre>
  </td>
  <td>
    Points & maxPoints only accessible when reviewsDone true and noReviews false<br>
    Firstname & Lastname of Reviewer is only accessible in non anonymous workshops
  </td>
</tr>
<tr>
  <td>Get given reviews and submissions that need attention<br><code>/student/todos</code></td>
  <td>GET</td>
  <td></td>
  <td>
<pre lang=json>
{
  "ok": &lt;b>,
  "reviews": [
    {
      "id": &lt;i>, "done": &lt;b>,
      "deadline": &lt;d>, "title": &lt;s>,
      "firstname": &lt;s>, "lastname": &lt;s>,
      "submission": &lt;i>, "workshopName": &lt;s>
    }, ..
  ],
  "submissions": [
    {
      "id": &lt;i>, "workshopName": &lt;s>
    }
  ]
}
</pre>
  </td>
  <td>
    Id property in "submissions" relates to the workshop Id
  </td>
</tr>
</tbody>
</table>

#### Submissions & Reviews

<table>
<thead>
<tr>
<th>Route</th>
<th>Method</th>
<th>Input</th>
<th>Output</th>
<th>Notes</th>
</tr>
</thead>
<tbody>
<tr>
  <td>Add submission<br><code>/submission/{workshop_id}</code></td>
  <td>POST</td>
  <td>
<pre lang=json>
{
  "title": &lt;s>, "comment": &lt;s>,
  "attachments": [&lt;i>, ..]
}
</pre>
  </td>
  <td>
<pre lang=json>
{
  "ok": &lt;b>,
  "id": &lt;i>
}
</pre>
  </td>
  <td>
    Attachments is a list of attachment Ids
  </td>
</tr>
<tr>
  <td>Get submission<br><code>/submission/{submission_id}</code></td>
  <td>GET</td>
  <td></td>
  <td>
Student (Own submission) & Teacher
<pre lang=json>
{
  "ok": &lt;b>,
  "title": &lt;s>, "comment": &lt;s>,
  "attachments": [
    { "id": &lt;i>, "title": &lt;s>}, ..
  ],
  "locked": &lt;b>, "date": &lt;d>,
  "firstname": &lt;s>, "lastname": &lt;s>,
  "reviewsDone": &lt;b>, "noReviews": &lt;b>,
  "points": &lt;f>, "maxPoints": &lt;f>,
  "reviews": [
    {
      "id": &lt;i>, "firstname": &lt;s>,
      "lastname": &lt;s>, "feedback": &lt;s>,
      "points": [
        {
          "id": &lt;i>, "type": &lt;sp>,
          "title": &lt;s>, "content: &lt;s>,
          "points": &lt;f>, "weight": &lt;f>
        }
      ]
    }
  ]
}
</pre>
<br>
Student (Other student submission)
<pre lang=json>
{
  "ok": &lt;b>,
  "title": &lt;s>, "comment": &lt;s>,
  "attachments": [
    { "id": &lt;i>, "title": &lt;s>}, ..
  ],
  "criteria": [
    {
       "id": &lt;i>, "type": &lt;sp>,
       "title": &lt;s>, "content": &lt;s>,
       "weight": &lt;f>
    }, ..
  ]
}
</pre>
<br>
  </td>
  <td>
    Reviewers first- & lastname are not available for students in anonymous workshops
  </td>
</tr>
<tr>
  <td>Update submission<br><code>/submission/{submission_id}</code></td>
  <td>PUT</td>
  <td>
<pre lang=json>
{
  "title": &lt;s>, "comment": &lt;s>,
  "attachments": [&lt;i>, ..]
}
</pre>
  </td>
  <td>
<pre lang=json>
{
  "ok": &lt;b>
}
</pre>
  </td>
  <td>
    Attachments is a list of attachment Ids
  </td>
</tr>
<tr>
  <td>Add attachment<br><code>/submission/upload</code></td>
  <td>POST</td>
  <td>Form data with key "file" containing the file and key "title" with a backup filename</td>
  <td>
<pre lang=json>
{
  "ok": &lt;b>,
  "attachment": {
    "id": &lt;i>, "title": &lt;s>
  }
}
</pre>
  </td>
  <td>
  </td>
</tr>
<tr>
  <td>Remove attachment<br><code>/submission/remove/{att_id}</code></td>
  <td>DELETE</td>
  <td></td>
  <td>
<pre lang=json>
{
  "ok": &lt;b>
}
</pre>
  </td>
  <td>
  </td>
</tr>
<tr>
  <td>Download attachment<br><code>/submission/download/{att_id}</code></td>
  <td>GET</td>
  <td></td>
  <td>Filestream</td>
  <td>
  </td>
</tr>
<tr>
  <td>Update review<br><code>/review/{review_id}</code></td>
  <td>PUT</td>
  <td>
<pre lang=json>
{
  "feedback": &lt;s>, "points": [
    {
      "id": &lt;i>, "points": &lt;f>
    }
  ]
}
</pre>
  </td>
  <td>
<pre lang=json>
{
  "ok": &lt;b>,
  "id": &lt;i>
}
</pre>
  </td>
  <td>
    Points Id is received through Criteria property from <code>GET /submission/{submission_id}</code> <br>
    Grading with Points: <br>
    "point":      1.0 - 10.0<br>
    "grade":      1.0 (A) - 5.0 (F)<br>
    "percentage": 0.0 - 100.0<br>
    "truefalse":  0.0 False, 1.0 True
  </td>
</tr>
<tr>
  <td>Get review<br><code>/review/{review_id}</code></td>
  <td>GET</td>
  <td><td>
<pre lang=json>
{
  "ok": &lt;b>,
  "id": &lt;i>, "firstname": &lt;s>,
  "lastname": &lt;s>, "notSubmitted": &lt;b>,
  "feedback": &lt;s>, "points": [
    {
      "type": &lt;sp>, "title": &lt;s>,
      "content": &lt;s>, "points": &lt;f>,
      "weight": &lt;f>
    }
  ]
}
</pre>
  </td>
  <td>
   Reviewers first- & lastname are not available in anonymous workshops
  </td>
</tr>
</tbody>
</table>

#### Creating user accounts

Only possible with admin account (= username `admin`).

| Route            | Method | Input                                                        | Output      | Notes |
| ---------------- | ------ | ------------------------------------------------------------ | ----------- | ----- |
| `/users/student` | POST   | <pre lang=json>{<br />"username": \<s>, <br />"firstname": \<s>,<br />"lastname": \<s>,<br />"password": \<s>,<br />"group": \<s><br />}</pre> | Status Code |       |
| `/users/teacher` | POST   | <pre lang=json>{<br />"username": \<s>, <br />"firstname": \<s>,<br />"lastname": \<s>,<br />"password": \<s><br />}</pre> | Status Code |       |



### 📖 Help

* [Windows] Build shows `libssl-1_1-x64.dll` not found

  Install OpenSSL add it to the path

  ```
  choco install openssl
  ```

  ```
  scoop install openssl
  ```

  

* [Windows] Problems with `libmysql.dll` (in binary or Diesel CLI)

  Install the C-Connector from [here](https://downloads.mysql.com/archives/c-c/) to a directory and it to the path like this:

  ```
  setx MYSQLCLIENT_LIB_DIR "<your path>"	
  ```

  Also put `libmysql.dll` from the C-Connector to the folder of the Diesel binary in `C:\Users\<user>\.cargo\bin` if you still encounter problems with the Diesel CLI.

  

* [Windows] Problems with connecting to MySQL databases (binary or Diesel CLI)

  For now, Diesel on Windows does not support `SSL` for MySQL. In order to use it, one must disable SSL on the Server.

  ```
  sudo nano /etc/mysql/mysql.conf.d/mysqld.cnf
  ```

  Now add following command

  ```
  [mysqld]
  ...
  skip_ssl
  ...
  ```

  

* Migrations with Diesel CLI

  If you want to run the migrations with the CLI (is not needed/recommended) you can run

  ```
  diesel setup --database-url "mysql://root@127.0.0.1:3306/iprp"
  ```

  ```
  diesel migration run --database-url "mysql://root@127.0.0.1:3306/iprp"
  ```

  

* [Windows] Building Rust-Crypto (dependency) fails on MinGW

  Set environment variable `CC=gcc`

  [Source](https://github.com/DaGenix/rust-crypto/issues/369)