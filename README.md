# Backend

## Peer-Review-Platform

### 📦 Requirements 

* Rust (Nightly)
* MySQL

### :rocket: Run

For convenience checkout the latest releases.

If you want to run it manually use 

```
cargo run
```

### 🗺️ API

Type Definitions:

* `<s>` - String
  * `<sr>` - "teacher" | "student"
  * `<sp>` - "point" | "grade" | "percentage" | "truefalse"
* `<b>` - Boolean
* `<i>` - Integer
* `<f>` - Float
* `<d>` - Date

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
<tr>
  <td>Show all workshops<br><code>/teacher/workshops</code></td>
  <td>GET</td>
  <td></td>
  <td>
<pre lang=json>
{ 
  workshops: [ 
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
    "teachers": [ { "id": &lt;i>, "firstname": &lt;s>, "lastname": &lt;s>}, .. ],
    "students": [ { "id": &lt;i>, "firstname": &lt;s>, "lastname": &lt;s>, "group": &lt;s>,
      "submissions": [ {"id": &lt;i>, "date": &lt;d>, "title": &lt;s>, "reviewsDone": &lt;b>,
      "studentid": &lt;i>, "noReviews": &lt;b>,
      "points": &lt;f>, "maxPoints": &lt;f>}, ..] 
  } ], .. }
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
<pre lang=json>
{ 
  "id": &lt;i>
}
</pre>
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
  <td>Get Student Id fron Name<br><code>/teacher/search/student</code></td>
  <td>GET</td>
  <td>
<pre lang=json>
{ 
  "firstname": &lt;s>, "lastname": &lt;s>
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
  <td></td>
</tr>
<tr>
  <td>Get all Students from a Group<br><code>/teacher/search/student</code></td>
  <td>GET</td>
  <td>
<pre lang=json>
{ 
  "group": &lt;s>
}
</pre>
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
</thead>
</table>


#### old

| Route    | Method | Input             | Output | Notes |
| -------- | ------ | ----------------- | ------ | ----- |
| `/login` | POST | Basic Auth Header | <pre lang=json>{<br />"id": \<i>,  "role": \<sr> <br />}</pre> | Also returns a Session Cookie! |
| `/logout` | POST | Session Cookie | Status Code | |



<table>
<thead>
<tr>
<th>abc</th>
<th>def</th>
</tr>
<tr>
<td>
    <pre lang=json>
    	a b c
    </pre>
</td>
<td></td>
</td>
</thead>
</table>



#### Creating user accounts

Only possible with admin account (= username `admin`).

| Route            | Method | Input                                                        | Output      | Notes |
| ---------------- | ------ | ------------------------------------------------------------ | ----------- | ----- |
| `/users/student` | POST   | <pre lang=json>{<br />"username": \<s>, <br />"firstname": \<s>,<br />"lastname": \<s>,<br />"password": \<s>,<br />"group": \<s><br />}</pre> | Status Code |       |
| `/users/teacher` | POST   | <pre lang=json>{<br />"username": \<s>, <br />"firstname": \<s>,<br />"lastname": \<s>,<br />"password": \<s><br />}</pre> | Status Code |       |

#### 



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