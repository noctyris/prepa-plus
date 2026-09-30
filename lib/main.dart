import 'package:flutter/material.dart';
import 'package:dynamic_color/dynamic_color.dart';

import 'api/client.dart';
import 'api/parse.dart';
import 'login.dart';
import 'notes.dart';

void main() {
  runApp(const PrepaHubApp());
}

class PrepaHubApp extends StatelessWidget {
  const PrepaHubApp({super.key});

  @override
  Widget build(BuildContext context) {
    return DynamicColorBuilder(
      builder: (lightDynamic, darkDynamic) {
        ColorScheme? light = lightDynamic;
        ColorScheme? dark = darkDynamic;
        light ??= ColorScheme.fromSeed(seedColor: const Color(0xFF3F51B5));
        dark ??= ColorScheme.fromSeed(
          seedColor: const Color(0xFF3F51B5),
          brightness: Brightness.dark,
        );
        return MaterialApp(
          title: 'PrepaHub',
          theme: ThemeData(colorScheme: light, useMaterial3: true),
          darkTheme: ThemeData(colorScheme: dark, useMaterial3: true),
          home: const RootPage(),
        );
      },
    );
  }
}

class RootPage extends StatefulWidget {
  const RootPage({super.key});

  @override
  State<RootPage> createState() => _RootPageState();
}

class _RootPageState extends State<RootPage> {
  bool loading = true;
  List<Semaine>? semaines;

  @override
  void initState() {
    super.initState();
    _autoLogin();
  }

  Future<void> _autoLogin() async {
    final creds = await loadCreds();
    if (!mounted) return;
    if (creds == null) {
      setState(() => loading = false);
      return;
    }
    try {
      final s = await getNotes(creds.username, creds.password);
      if (!mounted) return;
      setState(() => semaines = s);
    } catch (_) {
      await clearCreds();
      if (!mounted) return;
      setState(() => loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (loading) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }
    if (semaines == null) {
      return LoginPage(onSuccess: (s) => setState(() => semaines = s));
    }
    return NotesPage(semaines: semaines!);
  }
}
