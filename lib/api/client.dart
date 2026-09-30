import 'package:http/http.dart' as http;
import 'package:html/parser.dart' show parse;
import 'package:shared_preferences/shared_preferences.dart';
import 'dart:convert';
import 'parse.dart';

const base = 'https://cpgedupuydelome.prepas-plus.fr';
const loginUrl = '$base/account/login/';

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

Future<List<Semaine>> getNotes(String username, String password) async {
  final client = http.Client();

  final page = await client.get(Uri.parse(loginUrl));
  if (page.statusCode != 200) {
    throw Exception('Impossible d\'accéder à la page de connexion');
  }
  final csrf = parse(page.body)
      .querySelector('input[name="csrfmiddlewaretoken"]')
      ?.attributes['value'];
  if (csrf == null) throw Exception('Token CSRF introuvable');

  final login = await client.post(
    Uri.parse(loginUrl),
    headers: {'Referer': loginUrl},
    body: {
      'csrfmiddlewaretoken': csrf,
      'login_view-current_step': 'auth',
      'auth-username': username,
      'auth-password': password,
    },
    encoding: Encoding.getByName('utf-8'),
  );
  if (login.headers['location']?.contains('account/login') == true) {
    throw Exception('Échec de connexion');
  }

  final notesPage = await client.get(Uri.parse('$base/colles/mes_notes'));
  return parseNotes(notesPage.body);
}
