import 'package:app/application_state.dart';
import 'package:app/generated/moneyview.pbgrpc.dart';
import 'package:app/ui/transaction_widget.dart';
import 'package:file_picker/file_picker.dart';
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
          isEmpty = false;
        });
      });
    }
    return Center(
        child: ListView(
      children: [
        ElevatedButton.icon(
          onPressed: () async {
            FilePickerResult? result = await FilePicker.platform.pickFiles(
              type: FileType.custom,
              allowedExtensions: ['mta'],
            );
            if (result != null) {
              String content = String.fromCharCodes(result.files.single.bytes!);
              var response = await appState.moneyViewClient
                  .sendTextData(TextRequest(data: content));
              setState(() {
                transactions = response.transactions;
              });
            }
          },
          label: Text('Open'),
        ),
        for (var transaction in transactions)
          TransactionWidget(transaction: transaction),
      ],
    ));
  }
}
