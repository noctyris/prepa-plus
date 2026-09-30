import 'package:flutter/material.dart';
import 'api/parse.dart';
//import 'main.dart';

class NotesPage extends StatelessWidget {
  final List<Semaine> semaines;
  const NotesPage({super.key, required this.semaines});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Mes notes')),
      body: ListView.builder(
        itemCount: semaines.length,
        itemBuilder: (context, i) {
          final s = semaines[i];
          return Card(
            child: ListTile(
              title: Text('Semaine ${s.numero ?? '?'} · ${s.debut} → ${s.fin}'),
              subtitle: Text('${s.notes.length} colles'),
            ),
          );
        },
      ),
    );
  }
}
