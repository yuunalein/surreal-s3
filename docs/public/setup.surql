DEFINE FUNCTION OVERWRITE fn::install_s3($version: option<string>, $db_url: option<string>) {
  LET $base_url = "https://github.com/yuunalein/surreal-s3/";

  LET $url = IF $version = NONE {
    $base_url + "releases/latest/download/s3.surli"
  } ELSE {
    $base_url + "releases/download/" + $version + "/s3.surli"
  };

  LET $filename = IF $version = NONE {
    "s3-latest.surli"
  } ELSE {
    "s3-" + $version + ".surli"
  };

  DEFINE BUCKET IF NOT EXISTS s3_module_bucket BACKEND "memory";

  LET $file = type::file("s3_module_bucket", $filename);

  LET $data = http::get($url);
  $file.put($data);

  LET $db_base_url = IF $db_url = NONE {
    "http://127.0.0.1:8000"
  } ELSE {
    $db_url
  };

  LET $ver = http::get($db_base_url + "/version");
  LET $parts = string::split(string::split($ver, "-")[1], ".");
  LET $major = <int>$parts[0];
  LET $minor = <int>$parts[1];

  IF $major > 3 OR ($major = 3 AND $minor >= 3) {
    eval::surql('DEFINE MODULE OVERWRITE mod::s3 FROM f"s3_module_bucket:/' + $filename + '" UNSIGNED;');
  } ELSE {
    eval::surql('DEFINE MODULE OVERWRITE mod::s3 AS f"s3_module_bucket:/' + $filename + '";');
  };
};