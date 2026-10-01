import 'dart:async';

import 'package:html/parser.dart' show parse;
import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';

import 'parse.dart';

const base = 'https://cpgedupuydelome.prepas-plus.fr';
const loginUrl = '$base/account/login/';
const notesUrl = '$base/colles/mes_notes';

class AuthException implements Exception {
  final String message;
  const AuthException(this.message);
  @override
  String toString() => message;
}

class NetworkException implements Exception {
  final String message;
  const NetworkException(this.message);
  @override
  String toString() => message;
}

class Creds {
  final String username;
  final String password;
  Creds({required this.username, required this.password});
}

Future<Creds?> loadCreds() async {
  final prefs = await SharedPreferences.getInstance();
  final user = prefs.getString('username');
  final pass = prefs.getString('password');
  if (user == null || pass == null) return null;
  return Creds(username: user, password: pass);
}

Future<void> saveCreds(Creds creds) async {
  final prefs = await SharedPreferences.getInstance();
  await prefs.setString('username', creds.username);
  await prefs.setString('password', creds.password);
}

Future<void> clearCreds() async {
  final prefs = await SharedPreferences.getInstance();
  await prefs.remove('username');
  await prefs.remove('password');
}

class _Session {
  final http.Client _client = http.Client();
  final Map<String, String> _cookies = {};
  static const _timeout = Duration(seconds: 20);

  String? cookie(String name) => _cookies[name];
  void close() => _client.close();

  Future<http.Response> get(Uri url) => _send('GET', url);

  Future<http.Response> postForm(Uri url, Map<String, String> form,
          {Map<String, String>? headers}) =>
      _send('POST', url, form: form, headers: headers);

  Future<http.Response> _send(String method, Uri url,
      {Map<String, String>? form, Map<String, String>? headers}) async {
    var m = method;
    var u = url;
    var f = form;
    for (var hops = 0; hops < 10; hops++) {
      final req = http.Request(m, u)..followRedirects = false;
      req.headers['User-Agent'] = 'Mozilla/5.0 (Linux; Android) PrepaHub';
      if (headers != null) req.headers.addAll(headers);
      if (_cookies.isNotEmpty) {
        req.headers['Cookie'] =
            _cookies.entries.map((e) => '${e.key}=${e.value}').join('; ');
      }
      if (f != null) req.bodyFields = f;

      final res = await http.Response.fromStream(
          await _client.send(req).timeout(_timeout));
      _storeCookies(res);

      final loc = res.headers['location'];
      if ({301, 302, 303, 307, 308}.contains(res.statusCode) && loc != null) {
        u = u.resolve(loc);
        if (res.statusCode != 307 && res.statusCode != 308) {
          m = 'GET';
          f = null;
        }
        continue;
      }
      return res;
    }
    throw const NetworkException('Trop de redirections');
  }

  void _storeCookies(http.Response res) {
    final raw = res.headers['set-cookie'];
    if (raw == null) return;
    for (final c in raw.split(RegExp(r',(?=\s*[^;,=\s]+=)'))) {
      final parts = c.split(';');
      final kv = parts.first.trim();
      final i = kv.indexOf('=');
      if (i <= 0) continue;
      final name = kv.substring(0, i);
      final value = kv.substring(i + 1);
      final expired =
          parts.skip(1).any((p) => p.trim().toLowerCase() == 'max-age=0');
      if (value.isEmpty || expired) {
        _cookies.remove(name);
      } else {
        _cookies[name] = value;
      }
    }
  }
}

Future<List<Semaine>> getNotes(String username, String password) async {
  final session = _Session();
  try {
    final page = await session.get(Uri.parse(loginUrl));
    if (page.statusCode != 200) {
      throw NetworkException(
          'Page de connexion inaccessible (${page.statusCode})');
    }
    final csrf = parse(page.body)
            .querySelector('input[name="csrfmiddlewaretoken"]')
            ?.attributes['value'] ??
        session.cookie('csrftoken');
    if (csrf == null) throw const NetworkException('Token CSRF introuvable');

    final login = await session.postForm(
      Uri.parse(loginUrl),
      {
        'csrfmiddlewaretoken': csrf,
        'login_view-current_step': 'auth',
        'auth-username': username,
        'auth-password': password,
      },
      headers: {'Referer': loginUrl},
    );
    if (login.statusCode >= 400) {
      throw NetworkException('Erreur du serveur (${login.statusCode})');
    }
    final loginDoc = parse(login.body);
    if (loginDoc.querySelector('input[name="token-otp_token"]') != null) {
      throw const AuthException(
          "L'authentification à deux facteurs n'est pas prise en charge");
    }
    if (loginDoc.querySelector('input[name="auth-password"]') != null) {
      throw const AuthException('Identifiant ou mot de passe incorrect');
    }

    final res = await session.get(Uri.parse(notesUrl));
    if (res.statusCode != 200) {
      throw NetworkException('Erreur du serveur (${res.statusCode})');
    }
    if (parse(res.body).querySelector('input[name="auth-password"]') != null) {
      throw const AuthException('Session refusée par le serveur');
    }
    return parseNotes(res.body);
  } on AuthException {
    rethrow;
  } on NetworkException {
    rethrow;
  } on TimeoutException {
    throw const NetworkException('Délai dépassé, serveur injoignable');
  } catch (_) {
    throw const NetworkException('Réseau indisponible');
  } finally {
    session.close();
  }
}
