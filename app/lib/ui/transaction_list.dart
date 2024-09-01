import 'package:app/application_state.dart';
import 'package:app/generated/moneyview.pbgrpc.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class TransactionList extends StatefulWidget {
  const TransactionList({
    super.key,
  });

  @override
  State<TransactionList> createState() => _TransactionListState();
}

class _TransactionListState extends State<TransactionList> {
  List<Transaction> transactions = [];
  bool isEmpty = true;

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<ApplicationState>();
    if (isEmpty) {
    appState.moneyViewClient.getAllTransactions(Empty()).then((response) {
      setState(() {
        transactions = response.transactions;
        isEmpty=false;
      });
    });
    }
    return Center(
        child: Column(
      children: [
        for (var transaction in transactions)
          ListTile(
            leading: Icon(Icons.favorite),
            title: Text(transaction.id),
          ),
      ],
    ));
  }
}
