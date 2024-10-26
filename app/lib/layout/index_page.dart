import 'package:app/ui/partner_balance.dart';
import 'package:app/ui/tag_balance.dart';
import 'package:app/ui/tag_manager.dart';
import 'package:app/ui/transaction_list.dart';
import 'package:flutter/material.dart';

class IndexPage extends StatefulWidget {
  @override
  State<IndexPage> createState() => _IndexPageState();
}

class _IndexPageState extends State<IndexPage> {
  var selectedIndex = 0;

  @override
  Widget build(BuildContext context) {
    Widget page;
    switch (selectedIndex) {
      case 0:
        page = TransactionList();
      case 1:
        page = PartnerBalanceWidget();
      case 2:
        page = TagBalanceWidget();
      case 3:
        page = TagManager();
      default:
        throw UnimplementedError('no widget for $selectedIndex');
    }

    return LayoutBuilder(builder: (context, constraints) {
      return Scaffold(
          body: Row(children: [
        SafeArea(
          child: Navigation(constraints),
        ),
        Expanded(
          child: Container(
            color: Theme.of(context).colorScheme.primaryContainer,
            child: page,
          ),
        )
      ]));
    });
  }

  NavigationRail Navigation(BoxConstraints constraints) {
    return NavigationRail(
      extended: constraints.maxWidth >= 600,
      destinations: [
        NavigationRailDestination(
          icon: Icon(Icons.transform),
          label: Text('Transaktionen'),
        ),
        NavigationRailDestination(
          icon: Icon(Icons.account_balance_wallet),
          label: Text('Partner Balance'),
        ),
        NavigationRailDestination(
          icon: Icon(Icons.account_balance_wallet),
          label: Text('Tag Balance'),
        ),
        NavigationRailDestination(
          icon: Icon(Icons.account_balance_wallet),
          label: Text('Tag Manager'),
        ),
      ],
      selectedIndex: selectedIndex,
      onDestinationSelected: (value) {
        setState(() {
          selectedIndex = value;
        });
      },
    );
  }
}
