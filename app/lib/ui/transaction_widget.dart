import 'package:app/generated/moneyview.pbgrpc.dart';
import 'package:flutter/material.dart';

class TransactionWidget extends StatelessWidget {
  const TransactionWidget({
    super.key,
    required this.transaction,
  });

  final Transaction transaction;

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: EdgeInsets.all(16.0),
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Transaction ID: ${transaction.id}',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            SizedBox(height: 8.0),
            Text(
                'Date: ${DateTime.fromMillisecondsSinceEpoch(transaction.date.toInt() * 1000).toLocal()}'),
            SizedBox(height: 8.0),
            Text('Total Amount: ${transaction.totalAmount.toStringAsFixed(2)}'),
            SizedBox(height: 8.0),
            Text('Description: ${transaction.description}'),
            SizedBox(height: 8.0),
            Text(
                'Balance After Transaction: ${transaction.balanceAfterTransaction.toStringAsFixed(2)}'),
          ],
        ),
      ),
    );
  }
}
