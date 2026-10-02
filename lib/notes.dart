import 'package:flutter/material.dart';

import 'api/parse.dart';

String _fmt(double? v) {
  if (v == null) return '—';
  return v.toStringAsFixed(2).replaceAll(RegExp(r'\.?0+$'), '').replaceAll('.', ',');
}

class NotesPage extends StatelessWidget {
  final List<Semaine> semaines;
  final Future<void> Function() onRefresh;
  final VoidCallback onLogout;

  const NotesPage({
    super.key,
    required this.semaines,
    required this.onRefresh,
    required this.onLogout,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: RefreshIndicator(
        onRefresh: onRefresh,
        child: CustomScrollView(
          physics: const AlwaysScrollableScrollPhysics(),
          slivers: [
            SliverAppBar.large(
              title: const Text('Mes notes'),
              actions: [
                IconButton(
                  tooltip: 'Se déconnecter',
                  icon: const Icon(Icons.logout),
                  onPressed: onLogout,
                ),
              ],
            ),
            if (semaines.isEmpty)
              const SliverFillRemaining(
                hasScrollBody: false,
                child: Center(child: Text('Aucune note pour le moment')),
              )
            else
              SliverPadding(
                padding: const EdgeInsets.fromLTRB(16, 8, 16, 24),
                sliver: SliverList.separated(
                  itemCount: semaines.length,
                  separatorBuilder: (_, _) => const SizedBox(height: 12),
                  itemBuilder: (context, i) =>
                      _SemaineCard(key: ValueKey(i), semaine: semaines[i], expanded: i == semaines.length - 1),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _SemaineCard extends StatelessWidget {
  final Semaine semaine;
  final bool expanded;
  const _SemaineCard({super.key, required this.semaine, required this.expanded});

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    final cs = Theme.of(context).colorScheme;
    final n = semaine.notes.length;
    final dates = semaine.fin.isEmpty
        ? semaine.debut
        : '${semaine.debut} → ${semaine.fin}';
    return Card.filled(
      margin: EdgeInsets.zero,
      clipBehavior: Clip.antiAlias,
      child: ExpansionTile(
        shape: const Border(),
        collapsedShape: const Border(),
        initiallyExpanded: expanded,
        title: Text('Semaine ${semaine.numero ?? '?'}', style: tt.titleMedium),
        subtitle: Text('$dates · $n colle${n > 1 ? 's' : ''}',
            style: tt.bodySmall?.copyWith(color: cs.onSurfaceVariant)),
        childrenPadding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
        children: [
          if (semaine.notes.isEmpty)
            Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Text('Pas de note cette semaine',
                  style: tt.bodyMedium?.copyWith(color: cs.onSurfaceVariant)),
            )
          else
            for (final note in semaine.notes) _NoteTile(note: note),
        ],
      ),
    );
  }
}

class _NoteTile extends StatelessWidget {
  final Note note;
  const _NoteTile({required this.note});

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    final cs = Theme.of(context).colorScheme;

    Color bg = cs.secondaryContainer;
    Color fg = cs.onSecondaryContainer;
    if (note.note != null && note.moyenne != null) {
      if (note.note! >= note.moyenne!) {
        bg = cs.primaryContainer;
        fg = cs.onPrimaryContainer;
      } else {
        bg = cs.errorContainer;
        fg = cs.onErrorContainer;
      }
    }

    final r = note.rang;
    final chips = <String>[
      if (r.rang != null) 'Rang ${r.rang}${r.total != null ? '/${r.total}' : ''}',
      if (note.moyenne != null) 'Moy. ${_fmt(note.moyenne)}',
      if (note.ecartType != null) 'σ ${_fmt(note.ecartType)}',
    ];

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(note.matiere, style: tt.titleSmall),
                if (note.professeur.isNotEmpty)
                  Text(note.professeur,
                      style: tt.bodySmall?.copyWith(color: cs.onSurfaceVariant)),
                if (chips.isNotEmpty) ...[
                  const SizedBox(height: 8),
                  Wrap(
                    spacing: 8,
                    runSpacing: 4,
                    children: [
                      for (final c in chips)
                        Chip(
                          label: Text(c),
                          visualDensity: VisualDensity.compact,
                          materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                        ),
                    ],
                  ),
                ],
              ],
            ),
          ),
          const SizedBox(width: 12),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
            decoration: BoxDecoration(
              color: bg,
              borderRadius: BorderRadius.circular(16),
            ),
            child: Text(_fmt(note.note),
                style: tt.titleLarge?.copyWith(color: fg)),
          ),
        ],
      ),
    );
  }
}
