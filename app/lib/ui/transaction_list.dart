import 'package:app/application_state.dart';
import 'package:app/generated/moneyview.pbgrpc.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart'; // For date formatting
import 'package:provider/provider.dart';

class TransactionList extends StatefulWidget {
  const TransactionList({super.key});

  @override
  State<TransactionList> createState() => _TransactionListState();
}

class _TransactionListState extends State<TransactionList> {
  List<Transaction> transactions = [];
  List<Transaction> filteredTransactions = [];
  bool isEmpty = true;
  bool sortAscending = true;
  int sortColumnIndex = 0;
  int rowsPerPage = PaginatedDataTable.defaultRowsPerPage;
  String filterText = '';

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<ApplicationState>();

    // Load transactions only once when the table is initially empty
    if (isEmpty) {
      appState.moneyViewClient.getAllTransactions(Empty()).then((response) {
        setState(() {
          transactions = response.transactions;
          filteredTransactions =
              List.from(transactions); // Initially all transactions
          isEmpty = false;
        });
      });
    }

    // Sorting logic
    void _sort<T>(Comparable<T> Function(Transaction t) getField,
        int columnIndex, bool ascending) {
      setState(() {
        sortColumnIndex = columnIndex;
        sortAscending = ascending;
        filteredTransactions.sort((a, b) {
          final Comparable<T> aValue = getField(a);
          final Comparable<T> bValue = getField(b);
          return ascending
              ? Comparable.compare(aValue, bValue)
              : Comparable.compare(bValue, aValue);
        });
      });
    }

    // Filter logic
    void _filterTransactions(String query) {
      setState(() {
        filterText = query;
        filteredTransactions = transactions.where((transaction) {
          return transaction.partnerName
                  .toLowerCase()
                  .contains(query.toLowerCase()) ||
              transaction.description
                  .toLowerCase()
                  .contains(query.toLowerCase()) ||
              transaction.tags.any(
                  (tag) => tag.toLowerCase().contains(query.toLowerCase()));
        }).toList();
      });
    }

    return Center(
      child: Column(
        children: [
          // Filter TextField
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: TextField(
              decoration: InputDecoration(
                labelText: 'Filter by Partner Name, Description or Tags',
                border: OutlineInputBorder(),
              ),
              onChanged: (value) {
                _filterTransactions(value);
              },
            ),
          ),
          // Button to open file
          ElevatedButton.icon(
            onPressed: () async {
              FilePickerResult? result = await FilePicker.platform.pickFiles(
                type: FileType.custom,
                allowedExtensions: ['mta'],
              );
              if (result != null) {
                String content =
                    String.fromCharCodes(result.files.single.bytes!);
                var response = await appState.moneyViewClient
                    .sendTextData(TextRequest(data: content));
                setState(() {
                  transactions = response.transactions;
                  filteredTransactions =
                      List.from(transactions); // Reset filter after new data
                });
              }
            },
            icon: Icon(Icons.file_open),
            label: Text('Open'),
          ),
          // Paginated DataTable wrapped in expanded scrollable container
          Expanded(
            child: SingleChildScrollView(
              child: SingleChildScrollView(
                scrollDirection:
                    Axis.horizontal, // Allow horizontal scrolling if necessary
                child: ConstrainedBox(
                  constraints: BoxConstraints(
                    maxWidth: MediaQuery.of(context)
                        .size
                        .width, // Set maximum width to screen width
                  ),
                  child: PaginatedDataTable(
                    header: Text('Transactions'),
                    rowsPerPage: rowsPerPage,
                    onRowsPerPageChanged: (value) {
                      setState(() {
                        rowsPerPage = value!;
                      });
                    },
                    sortAscending: sortAscending,
                    sortColumnIndex: sortColumnIndex,
                    columns: [
                      DataColumn(
                        label: Text('Description'),
                        onSort: (columnIndex, ascending) =>
                            _sort((t) => t.description, columnIndex, ascending),
                      ),
                      DataColumn(
                        label: Text('Partner Name'),
                        onSort: (columnIndex, ascending) =>
                            _sort((t) => t.partnerName, columnIndex, ascending),
                      ),
                      DataColumn(
                        label: Text('Total Amount (€)'),
                        numeric: true,
                        onSort: (columnIndex, ascending) =>
                            _sort((t) => t.totalAmount, columnIndex, ascending),
                      ),
                      DataColumn(
                        label: Text('Date'),
                        onSort: (columnIndex, ascending) =>
                            _sort((t) => t.date, columnIndex, ascending),
                      ),
                      DataColumn(
                        label: Text('Tags'),
                      ),
                    ],
                    source: _TransactionDataSource(filteredTransactions),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

// DataTable source class for paginated data
class _TransactionDataSource extends DataTableSource {
  final List<Transaction> transactions;

  _TransactionDataSource(this.transactions);

  @override
  DataRow? getRow(int index) {
    if (index >= transactions.length) return null;
    final transaction = transactions[index];

    // Convert date from days since 1/1/1970 to DateTime and format it
    final transactionDate =
        DateTime(1970, 1, 1).add(Duration(days: transaction.date.toInt()));
    final formattedDate = DateFormat('dd-MM-yyyy').format(transactionDate);

    return DataRow.byIndex(
      index: index,
      cells: [
        DataCell(
          Container(
            width: 200, // Set width to 25% of screen width
            child: Text(
              transaction.description,
              maxLines: 2, // Allow multiline in description
              overflow: TextOverflow.ellipsis, // Add ellipsis for long text
            ),
          ),
        ),
        DataCell(Text(transaction.partnerName)),
        DataCell(Text(
            '${transaction.totalAmount.toStringAsFixed(2)} €')), // Round to 2 decimals and add €
        DataCell(Text(formattedDate)),
        DataCell(
          Text(
            transaction.tags.join(', '), // Display tags as comma-separated list
          ),
        ),
      ],
    );
  }

  @override
  int get rowCount => transactions.length;

  @override
  bool get isRowCountApproximate => false;

  @override
  int get selectedRowCount => 0;
}
