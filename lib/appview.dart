import 'package:flutter/material.dart';

import 'api/parse.dart';
import 'notes.dart';
import 'colloscope.dart';

class AppViewPage extends StatelessWidget {
    final List <Semaine> semaines;
    final Future<void> Function() onRefresh;
    final VoidCallback onLogout;
    
    const AppViewPage({
        super.key,
        required this.semaines,
        required this.onRefresh,
        required this.onLogout,
    });

    @override
    Widget build(BuildContext context) {
        return Navigation(
            semaines:   semaines,
            onRefresh:  onRefresh,
            onLogout:   onLogout
        );
    }
}

class Navigation extends StatefulWidget {
  final List<Semaine> semaines;
  final Future<void> Function() onRefresh;
  final VoidCallback onLogout;

  const Navigation({
    super.key,
    required this.semaines,
    required this.onRefresh,
    required this.onLogout,
  });

  @override
  State<Navigation> createState() => _NavigationState();
}

class _NavigationState extends State<Navigation> {
  int currentPageIndex = 0;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      bottomNavigationBar: NavigationBar(
        onDestinationSelected: (int index) {
          setState(() {
            currentPageIndex = index;
          });
        },
        selectedIndex: currentPageIndex,
        destinations: const <Widget>[
          NavigationDestination(
            selectedIcon: Icon(Icons.pie_chart),
            icon: Icon(Icons.home_outlined),
            label: 'Notes',
          ),
          NavigationDestination(
//            icon: Badge(label: Text('2'), child: Icon(Icons.calendar_month)),
            icon: Icon(Icons.calendar_month),
            label: 'Colloscope',
          ),
          NavigationDestination(
            icon: Icon(Icons.abc),
            label: '',
          ),
        ],
      ),
      body: <Widget>[
        /// Notes page
        NotesPage(
            semaines:   widget.semaines,
            onRefresh:  widget.onRefresh,
            onLogout:   widget.onLogout,
        ),
        
        /// Colloscope
        ColloscopePage(),
        
        /// Messages page
        Text("Rien")
        ][currentPageIndex],
    );
  }
}
