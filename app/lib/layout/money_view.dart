import 'package:app/application_state.dart';
import 'package:app/layout/index_page.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class MoneyView extends StatelessWidget {
  const MoneyView({super.key});

  @override
  Widget build(BuildContext context) {
    final app_state = ApplicationState();
    return ChangeNotifierProvider(
      create: (context) => app_state,
      child: MaterialApp(
        title: 'Namer App',
        theme: ThemeData(
          useMaterial3: true,
          colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        ),
        home: IndexPage(),
      ),
    );
  }
}
